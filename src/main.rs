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
    lower: u64,

    /// Upper bound on displayed percentile
    #[structopt(short, long, default_value="100")]
    upper: u64,

    /// Maximum number of percentile lines to display
    #[structopt(short, long, default_value="100")]
    max_lines: usize,

    /// Resolution of percentile display.
    #[structopt(short, long, default_value="1")]
    resolution: u64

}

fn main() -> Result<(), std::io::Error> {

    // Set options
    let args: Opt = Opt::from_args();

    if args.lower > args.upper {
        panic!("Lower percentile bound is greater than upper percentile bound");
    }


    // Read in data
    let file:File = File::open(args.input).expect("file not found");

    let lines: Vec<u64> = io::BufReader::new(file)
        .lines()
        .map(|line| {
            let l = line.unwrap();
            let l: Vec<&str> = l.split(" ").collect();
            if l.len() <= args.column {
                panic!("Error! Given column does not exist in data for line:\n---\n{0}\n----", l.clone()[0]);
            }
            l[args.column].to_owned().parse::<u64>().expect("Value was not parsable to an integer")
    }).collect();


    // Populate histogram
    let mut hist = Histogram::<u64>::new(3).expect("Unable to create histogram");

    for val in lines.iter() {
        hist.record(*val).expect("Value added to histogram is out of range");
    }


    // Print out the information
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    write_header(&hist, &mut stdout)?;


    let percentiles = construct_percentiles(&hist, 
        args.resolution,
        args.upper, 
        args.lower);

    write_percentiles(&percentiles, args.max_lines, &mut stdout)?;
    
    Ok(())
}

fn write_header<W: Write>(hist: &Histogram<u64>, writer: &mut W)
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
        writer.write_all(format!("Outlier(s) >= {0: >10.2}",
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

fn construct_percentiles(hist: &Histogram<u64>,
               resolution: u64,
               upper_bound: u64,
               lower_bound: u64) 
-> Vec<String> {

    let mut out: Vec<String> = Vec::new();
    for v in hist.iter_linear(resolution) {
        if lower_bound as f64 <= v.percentile() &&
           v.percentile() <= upper_bound as f64 && 
           v.count_at_value() != 0 {
            out.push(format!("{: >6.2} {: >10} {: >10}",
                v.percentile(),v.count_at_value(), v.value_iterated_to()));
        }
    } 

    out.reverse();
    out
}

fn write_percentiles<W: Write>(percentiles: &Vec<String>, 
                               max_lines: usize,
                               writer: &mut W)
-> Result<(), std::io::Error> {
    writer.write_all(format!("Percentile  count      value\n").as_ref())?;
    let line_count = if percentiles.len() < max_lines { percentiles.len() } else { max_lines };
    for l in percentiles[0..line_count].iter() {
        writer.write_all(format!("{}\n", l).as_ref())?;
    }   

    Ok(())
}
