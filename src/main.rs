use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::fs::File;
use hdrhistogram::iterators::{PickyIterator, HistogramIterator};
use structopt::StructOpt;
use hdrhistogram::Histogram;

#[derive(Debug, StructOpt)]
#[structopt(name = "qhist", about = "Simple historgraphic information")]
struct Opt {
    /// Path to file
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// The space delimited column to read data from (zero indexed)
    /// 
    /// The column of data may contain spaces any number of spaces 
    /// on either side of the u64 value. Default value is zero to allow
    /// for easy piping of data from STDIN.
    #[structopt(short, long, default_value="0")]
    column: usize,

    /// Lowest percentile to display
    #[structopt(short, long, default_value="0")]
    lower: u64,

    /// Highest percentile to display
    #[structopt(short, long, default_value="100")]
    upper: u64,

    /// Maximum number of percentile lines to display
    #[structopt(short, long, default_value="100")]
    max_lines: usize,

    /// Bucket size for percentile display.
    /// 
    /// When this option is supplied the iteration method to generate
    /// buckets is linear stepping at `--resolution` values. This stepping
    /// can, and will, lead to larger bucket counts which can invert 
    /// the scaling for the simple barchart. See `--bar-length` for more
    /// information.
    #[structopt(short, long)]
    resolution: Option<u64>,

    /// Do not print simple info block
    #[structopt(long)]
    no_info: bool,

    /// Do not print percentiles. Implies `--no-bars`.
    #[structopt(long)]
    no_percentiles: bool,

    /// Do not print simple bar chart
    #[structopt(long)]
    no_bars: bool,

    /// Max bar length
    /// 
    /// This option controls the number of tick marks in the longest
    /// bar of the chart. That is, the count for each value is normalized
    /// to [0,1) and multipled by `--bar-length`. When `--resolution` is
    /// given the bucket size is altered and no longer known at histogram
    /// creation. This may result in inverse scaling of the bar chart. When
    /// this occurs use smaller values for `--bar-length`
    #[structopt(short, long, default_value="100")]
    bar_length: f64,
}


fn main() -> Result<(), std::io::Error> {

    // Get options
    let args: Opt = Opt::from_args();

    // Some argument validation
    if args.lower > args.upper {
        panic!("Lower percentile bound is greater than upper percentile bound");
    }

    // Simpe barchart related argument dependencies.
    let no_bars = match args.no_percentiles {
        true => true,
        false => args.no_bars
    };
    let bar_length: f64 = match no_bars {
        true => 0.,
        false => args.bar_length
    };
        
        
    // Read in data
    let lines: Vec<u64>;
    if args.input == None {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();
        lines = read_data_from(stdin, args.column);
    } else {
        let file:File = File::open(args.input.unwrap()).expect("file not found");
        let file = io::BufReader::new(file);
        lines = read_data_from(file, args.column);

    }

    // Populate histogram
    let mut max_count: u64 = u64::MIN;
    let mut min_count: u64 = u64::MAX;
    let mut hist = Histogram::<u64>::new(3).expect("Unable to create histogram");

    for val in lines.iter() {
        hist.record(*val).expect("Value added to histogram is out of range");
        if hist.count_at(*val) > max_count { max_count = hist.count_at(*val); }
        else if hist.count_at(*val) < min_count { min_count = hist.count_at(*val); }
    }


    // Print out the information
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if !args.no_info {
        write_info_to(&mut stdout, &hist)?;
    }

    if !args.no_percentiles {
        let percentiles = match args.resolution {
            Some(resolution) => {
                construct_percentiles(&mut hist.iter_linear(resolution), 
                                       args.upper, 
                                       args.lower,
                                        max_count,
                                        min_count,
                                        bar_length)
            }
            None => {
                construct_percentiles(&mut hist.iter_recorded(), 
                                       args.upper, 
                                       args.lower,
                                        max_count,
                                        min_count,
                                        bar_length)
            }
        };

        write_percentiles_to(&mut stdout, &percentiles, args.max_lines, args.no_info)?;
    }
    
    Ok(())
}

/// Returns a vector containing the data pointed to by column and reader.
fn read_data_from<R: BufRead>(reader: R, column: usize) -> Vec<u64> {
    let lines: Vec<u64> = reader
        .lines()
        .map(|line| {
            let l = line.unwrap();
            let l: Vec<&str> = l.split_ascii_whitespace().collect();
            if l.len() <= column {
                panic!("Error! Given column does not exist in data for line:\n---\n{0}\n----", l.clone()[0]);
            }
            l[column].to_owned().parse::<u64>().expect(
                format!("Value ({0:#?}) at column {1} was not parsable to an integer!", l[column], column).as_ref())
    }).collect();
    lines
}


/// Prints simple histographic information to STDOUT
fn write_info_to<W: Write>(writer: &mut W, hist: &Histogram<u64>)
-> Result<(), std::io::Error> {
    writer.write_all(format!(
            "Samples: {0: >7}\n\
            Max:  {1: >10.2}\n\
            Min:  {2: >10.2}\n\
            Mean: {3: >10.2}\n\
            SD:   {4: >10.2}\n", 
            hist.len(),
            hist.highest_equivalent(hist.value_at_percentile(100.)), 
            hist.lowest_equivalent(hist.value_at_percentile(0.)),
            hist.mean(), 
            hist.stdev()).as_ref()
    )?;

    if (hist.mean() + 3. * hist.stdev()) <= hist.max() as f64{
        writer.write_all(format!("Outlier(s) >= {0: >10.2}\n",
            hist.mean() + 3. * hist.stdev()
        ).as_ref())?;
    }
    if (hist.mean() - 3. * hist.stdev()) >= hist.min() as f64{
        writer.write_all(format!("Outlier(s) <= {0: >10.2}\n",
            hist.mean() + 3. * hist.stdev()).as_ref()
        )?;
    }
    Ok(())
}

/// Constructs the percentile, and optionally the barchart, strings line by line
/// from a `HistogramIterator` which is returned as a vector of `String`s.
fn construct_percentiles<I: PickyIterator<u64>>(hist: &mut HistogramIterator<u64,I>,
                         upper_bound: u64,
                         lower_bound: u64,
                         max: u64,
                         min: u64,
                         bar_length: f64)
-> Vec<String> {

    let mut out: Vec<String> = Vec::new();
    //for v in hist.iter_linear(resolution) {
    for v in hist {
        if lower_bound as f64 <= v.percentile() &&
           v.count_since_last_iteration() != 0 && 
           v.percentile() <= upper_bound as f64  {
            out.push(format!("{: >6.2} {: >10} {: >10} {}",
                v.percentile(),
                v.value_iterated_to(), 
                v.count_since_last_iteration(),
                bar_string(v.count_since_last_iteration(), max, min, bar_length)));
        }
    } 
    out.reverse();
    out
}

/// Generate the scaled bar for the bucket represented by `val`
fn bar_string(val: u64, max: u64, min: u64, max_length: f64) -> String {
    let scaling = get_scaled(val as f64,
                                    max as f64,
                                    min as f64);
    let bar = "-".repeat( (scaling * max_length) as usize);
    bar
}

#[inline]
fn get_scaled(val: f64, max: f64, min: f64) -> f64 {
    (val - min) / (max - min)
}

/// Write the percentile strings to `writer`.
fn write_percentiles_to<W: Write>(writer: &mut W,
                                 percentiles: &Vec<String>, 
                                 max_lines: usize,
                                 no_info: bool)
-> Result<(), std::io::Error> {
    if !no_info {
        writer.write_all(format!("Percentile  bucket      count\n").as_ref())?;
    }
    let line_count = if percentiles.len() < max_lines { percentiles.len() } else { max_lines };
    for l in percentiles[0..line_count].iter() {
        writer.write_all(format!("{}\n", l).as_ref())?;
    }   

    Ok(())
}
