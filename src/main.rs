use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::fs::File;
use structopt::StructOpt;
use hdrhistogram::Histogram;

#[derive(Debug, StructOpt)]
#[structopt(name = "qhist", about = "description")]
struct Opt {
    /// Input file to read
    #[structopt(short,long)]
    input: PathBuf,

    /// Column of input data which should be operated on.
    #[structopt(short, long)]
    column: usize,

    /// Lower bound on displayed percentile
    #[structopt(short, long, default_value="0")]
    lower: f64,

    /// Upper bound on displayed percentile
    #[structopt(short, long, default_value="100")]
    upper: f64,

    /// Maximum number of percentile lines to display
    #[structopt(short, long, default_value="100")]
    max_lines: usize,

    /// Resolution of percentile display.
    #[structopt(short, long, default_value="1")]
    resolution: u64

}

fn main() {
   
    // Set options
    let opt: Opt = Opt::from_args();
    let column: usize = opt.column;
    let f_name: PathBuf = opt.input;
    let max_lines: usize = opt.max_lines;
    let upper_perc: f64  = opt.upper;
    let lower_perc: f64 = opt.lower;
    let reso: u64 = opt.resolution;

    if lower_perc > upper_perc {
        panic!("Lower percentile bound is greater than upper percentile bound");
    }


    // Read in data
    let file:File = File::open(f_name).expect("file not found");

    let lines: Vec<u64> = io::BufReader::new(file)
        .lines()
        .map(|line| {
            let l = line.unwrap();
            let l: Vec<&str> = l.split(" ").collect();
            if l.len() <= column {
                panic!("Error! Given column does not exist in data for line:\n---\n{0}\n----", l.clone()[0]);
            }
            l[column].to_owned().parse::<u64>().expect("Value was not parsable to an integer")
    }).collect();


    // Populate histogram
    let mut hist = Histogram::<u64>::new(3).expect("Unable to create histogram");

    for val in lines.iter() {
        hist.record(*val).expect("Value added to histogram is out of range");
    }


    // Print out the information
    println!("Samples: {0: >7}", hist.len());
    println!("Max:  {0: >10.2}", hist.highest_equivalent(hist.value_at_percentile(100.)));
    println!("Min:  {0: >10.2}", hist.lowest_equivalent(hist.value_at_percentile(0.)));
    println!("Mean: {0: >10.2}", hist.mean());
    println!("SD:   {0: >10.2}", hist.stdev());
    if (hist.mean() + 3. * hist.stdev()) <= hist.max() as f64{
        println!("Outlier(s) >= {0: >10.2}", hist.mean() + 3. * hist.stdev());
    }
    if (hist.mean() - 3. * hist.stdev()) >= hist.min() as f64{
        println!("Outlier(s) <= {0: >10.2}", hist.mean() + 3. * hist.stdev());
    }

    println!("Percentile  count      value");

    let mut out: Vec<String> = Vec::new();
    for v in hist.iter_linear(reso) {

        if lower_perc < v.percentile() &&
           v.percentile() <= upper_perc && 
           v.count_at_value() != 0 {
            out.push(format!("{: >6.2} {: >10} {: >10}",
                v.percentile(),v.count_at_value(), v.value_iterated_to()));
        }
    } 

    out.reverse();
    let line_count = if out.len() < max_lines { out.len() } else { max_lines };
    for l in out[0..line_count].iter() {
        println!("{}", l);
    }

}
