use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::fs::File;
use hdrhistogram::iterators::{PickyIterator, HistogramIterator};
use structopt::StructOpt;
use hdrhistogram::Histogram;

#[derive(Debug, StructOpt)]
#[structopt(name = "qhist", about = "Simple historgraphic information")]
struct Opt {
    /// Input file to read
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// Column of input data which should be operated on.
    #[structopt(short, long, default_value="0")]
    column: usize,

    /// Lower bound on displayed percentile
    #[structopt(short, long, default_value="0")]
    lower: u64,

    /// Upper bound on displayed percentile
    #[structopt(short, long, default_value="100")]
    upper: u64,

    /// Maximum number of percentile lines to display
    #[structopt(short, long, default_value="100")]
    max_lines: usize,

    /// Resolution of percentile display.
    #[structopt(short, long)]
    resolution: Option<u64>,

    /// Do not print info simple info block
    #[structopt(long)]
    no_info: bool,

    /// Do not print percentiles
    #[structopt(long)]
    no_percentiles: bool,
}


fn main() -> Result<(), std::io::Error> {

    // Set options
    let args: Opt = Opt::from_args();

    if args.lower > args.upper {
        panic!("Lower percentile bound is greater than upper percentile bound");
    }

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
    let mut hist = Histogram::<u64>::new(3).expect("Unable to create histogram");

    for val in lines.iter() {
        hist.record(*val).expect("Value added to histogram is out of range");
    }


    // Print out the information
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if !args.no_info {
        write_info_to(&mut stdout, &hist)?;
    }

    if !args.no_percentiles {
        let scaling: f64 = 1. / 9.;
        let percentiles = match args.resolution {
            Some(resolution) => {
                construct_percentiles(&mut hist.iter_linear(resolution), 
                                       args.upper, 
                                       args.lower,
                                        scaling)
            }
            None => {
                construct_percentiles(&mut hist.iter_recorded(), 
                                       args.upper, 
                                       args.lower,
                                        scaling)
            }
        };

        write_percentiles_to(&mut stdout, &percentiles, args.max_lines)?;
    }
    
    Ok(())
}

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

fn construct_percentiles<I: PickyIterator<u64>>(hist: &mut HistogramIterator<u64,I>,
                         upper_bound: u64,
                         lower_bound: u64,
                         scaling: f64) 
-> Vec<String> {

    let mut out: Vec<String> = Vec::new();
    //for v in hist.iter_linear(resolution) {
    for v in hist {
        if lower_bound as f64 <= v.percentile() &&
           v.count_since_last_iteration() != 0 && 
           v.percentile() <= upper_bound as f64  {
            let bar = "+".repeat( (v.count_since_last_iteration() as f64 * scaling) as usize);
            out.push(format!("{: >6.2} {: >10} {: >10} {}",
                v.percentile(),
                v.value_iterated_to(), 
                v.count_since_last_iteration(), bar));
        }
    } 
    out.reverse();
    out
}

fn write_percentiles_to<W: Write>(writer: &mut W,
                                 percentiles: &Vec<String>, 
                                 max_lines: usize)
-> Result<(), std::io::Error> {
    writer.write_all(format!("Percentile  bucket      count\n").as_ref())?;
    let line_count = if percentiles.len() < max_lines { percentiles.len() } else { max_lines };
    for l in percentiles[0..line_count].iter() {
        writer.write_all(format!("{}\n", l).as_ref())?;
    }   

    Ok(())
}
