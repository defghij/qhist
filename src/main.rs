use hdrhistogram::iterators::{HistogramIterator, PickyIterator};
use hdrhistogram::Histogram;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use structopt::StructOpt;

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
    #[structopt(short, long, default_value = "0")]
    column: usize,

    /// Lowest percentile to display
    #[structopt(short, long, default_value = "0")]
    lower: u64,

    /// Highest percentile to display
    #[structopt(short, long, default_value = "100")]
    upper: u64,

    /// Maximum number of percentile lines to display
    #[structopt(short, long, default_value = "100")]
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
    #[structopt(short, long, default_value = "100")]
    bar_length: f64,

    /// Number of decimal places of to keep for floating point input. Will garble integer input. 
    /// 
    /// This is used to convert the input from floating point into an integer to be operated on.
    /// Then used to convert back to a floating point for output. An input of `1.13` with `-s 2` 
    /// will be converted to `1.13 * 10^2 = 113` and processed. When output it will be reconverted
    /// to `113 * 10^2 = 1.13`. This is because the underlying library for creating the histogram
    /// does not suppor floating point values.
    #[structopt(short, long)]
    sig_figs: Option<f64>,
}

struct App {
    column: usize,
    upper: u64,
    lower: u64,
    max_count: u64,
    min_count: u64,
    max_lines: usize,
    no_info: bool,
    bar_length: f64,
    sig_figs: Option<f64>
}

fn main() -> Result<(), std::io::Error> {
    // Get options
    let args: Opt = Opt::from_args();

    // Some argument validation
    if args.lower > args.upper {
        panic!("Lower percentile bound is greater than upper percentile bound");
    }

    // Barchart related argument dependencies.
    let no_bars = match args.no_percentiles {
        true => true,
        false => args.no_bars,
    };
    let bar_length: f64 = match no_bars {
        true => 0.,
        false => args.bar_length,
    };

    let mut app: App = App {
        column: args.column,
        upper: args.upper,
        lower: args.lower, 
        max_count: u64::MIN, 
        min_count: u64::MAX, 
        max_lines: args.max_lines, 
        no_info: args.no_info,
        bar_length: bar_length, 
        sig_figs: args.sig_figs 
    };

    // Read in data
    let lines: Vec<u64>;
    if args.input == None {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();
        lines = read_data_from(stdin, &app);
    } else {
        let file: File = File::open(args.input.unwrap()).expect("file not found");
        let file = io::BufReader::new(file);
        lines = read_data_from(file, &app);
    }

    // Populate histogram
    let mut hist = Histogram::<u64>::new(3).expect("Unable to create histogram");

    for val in lines.iter() {
        hist.record(*val)
            .expect("Value added to histogram is out of range");
        if hist.count_at(*val) > app.max_count {
            app.max_count = hist.count_at(*val);
        } else if hist.count_at(*val) < app.min_count {
            app.min_count = hist.count_at(*val);
        }
    }

    // Print out the information
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    if !args.no_info {
        write_info_to(&mut stdout, &hist, app.sig_figs)?;
    }

    if !args.no_percentiles {
        let percentiles = match args.resolution {
            Some(resolution) => construct_percentiles(
                &mut hist.iter_linear(resolution), &app),
            None => construct_percentiles(
                &mut hist.iter_recorded(), &app),
        };

        write_percentiles_to(&mut stdout,
            &percentiles,
            app.max_lines,
            app.no_info,
        )?;
    }

    Ok(())
}

/// Returns a vector containing the data pointed to by column and reader.
fn read_data_from<R: BufRead>(reader: R, app: &App) -> Vec<u64> {
    let lines: Vec<u64> = reader
        .lines()
        .map(|line| {
            let l = line.unwrap();
            let l: Vec<&str> = l.split_ascii_whitespace().collect();
            if l.len() <= app.column {
                panic!(
                    "Error! Given column does not exist in data for line:\n---\n{0}\n----",
                    l.clone()[0]
                );
            }
            match app.sig_figs {
                Some(s) => {
                    // We have requested some number of significant figures, s, be maintained.
                    // This also assumes floating point input, c, was given.
                    // So the converted value c = (l[column] * 10 ^ s) as u64
                    let a: f64 = l[app.column].to_owned().parse::<f64>().expect(
                        format!(
                            "Value ({0:#?}) at column {1} was not parsable to a float!",
                            l[app.column], app.column
                        )
                        .as_ref()
                    ) as f64;
                    (a * f64::powf(10., s)) as u64
                },
                None => {
                    l[app.column].to_owned().parse::<u64>().expect(
                        format!(
                            "Value ({0:#?}) at column {1} was not parsable to an integer!",
                            l[app.column], app.column
                        )
                        .as_ref(),
                    )
                }
            }
        })
        .collect();
    lines
}

fn scale_per_sig_figs(value: f64, sig_figs: Option<f64>) -> f64 {
    match sig_figs {
        Some(s) => {
            let a: f64 = value;
            a / f64::powf(10., s)
        },
        None => value,
    }
}

/// Prints simple histographic information to STDOUT
fn write_info_to<W: Write>(writer: &mut W, hist: &Histogram<u64>, sig_figs: Option<f64>) -> Result<(), std::io::Error> {

    writer.write_all(
        format!(
            "Samples:  {0: >10.2}\n\
            Max:      {1: >10.2}\n\
            Min:      {2: >10.2}\n\
            Mean:     {3: >10.2}\n\
            SD:       {4: >10.2}\n",
            hist.len(),
            scale_per_sig_figs(hist.highest_equivalent(hist.value_at_percentile(100.)) as f64, sig_figs),
            scale_per_sig_figs(hist.lowest_equivalent(hist.value_at_percentile(0.)) as f64, sig_figs),
            scale_per_sig_figs(hist.mean(), sig_figs),
            scale_per_sig_figs(hist.stdev(), sig_figs)
        )
        .as_ref(),
    )?;

    let outliers_above = hist.mean() + 3. * hist.stdev();
    let outliers_below = hist.mean() - 3. * hist.stdev();

    if outliers_above <= hist.max() as f64 {
        writer.write_all(
            format!(
                "Outlier(s) >= {0: >5.2}\n",
                scale_per_sig_figs(outliers_above, sig_figs)
            )
            .as_ref(),
        )?;
    }
    if outliers_below >= hist.min() as f64 {
        writer.write_all(
            format!(
                "Outlier(s) <= {0: >5.2}\n",
                scale_per_sig_figs(outliers_below, sig_figs)
            )
            .as_ref(),
        )?;
    }
    Ok(())
}

/// Constructs the percentile, and optionally the barchart, strings line by line
/// from a `HistogramIterator` which is returned as a vector of `String`s.
fn construct_percentiles<I: PickyIterator<u64>>(
    hist: &mut HistogramIterator<u64, I>,
    app: &App
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    //for v in hist.iter_linear(resolution) {
    for v in hist {
        if app.lower as f64 <= v.percentile()
            && v.count_since_last_iteration() != 0
            && v.percentile() <= app.upper as f64
        {
            let deci_places: usize = match app.sig_figs {
                Some(s) => s as usize,
                None => 0,
            };

            out.push(format!(
                "{1: >6.2} {2: >10.0$} {3: >10} {4}",
                deci_places,
                v.percentile(),
                match app.sig_figs {
                    Some(s) => {
                        let a: f64 = v.value_iterated_to() as f64;
                        a / f64::powf(10., s)
                    },
                    None => v.value_iterated_to() as f64,
                },
                v.count_since_last_iteration(),
                bar_string(v.count_since_last_iteration(), app.max_count, app.min_count, app.bar_length)
            ));
        }
    }
    out.reverse();
    out
}

/// Generate the scaled bar for the bucket represented by `val`
fn bar_string(val: u64, max: u64, min: u64, max_length: f64) -> String {
    let scaling = get_scaled(val as f64, max as f64, min as f64);
    let bar = "-".repeat((scaling * max_length) as usize);
    bar
}

#[inline]
fn get_scaled(val: f64, max: f64, min: f64) -> f64 {
    (val - min) / (max - min)
}

/// Write the percentile strings to `writer`.
fn write_percentiles_to<W: Write>(
    writer: &mut W,
    percentiles: &Vec<String>,
    max_lines: usize,
    no_info: bool,
) -> Result<(), std::io::Error> {
    if !no_info {
        writer.write_all(format!("Percentile  Bucket      Count\n").as_ref())?;
    }
    let line_count = if percentiles.len() < max_lines {
        percentiles.len()
    } else {
        max_lines
    };
    for l in percentiles[0..line_count].iter() {
        writer.write_all(format!("{}\n", l).as_ref())?;
    }

    Ok(())
}
