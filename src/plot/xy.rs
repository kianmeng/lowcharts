use std::fmt;
use std::ops::Range;

use yansi::Color::{Blue, Red};

use crate::format::F64Formatter;
use crate::stats::Stats;

#[derive(Debug)]
pub struct XyPlot {
    x_axis: Vec<f64>,
    y_axis: Vec<f64>,
    width: usize,
    height: usize,
    stats: Stats,
    precision: Option<usize>,
}

impl XyPlot {
    pub fn new(width: usize, height: usize, stats: Stats, precision: Option<usize>) -> XyPlot {
        XyPlot {
            x_axis: Vec::with_capacity(width),
            y_axis: Vec::with_capacity(height),
            width,
            height,
            stats,
            precision,
        }
    }

    pub fn load(&mut self, vec: &[f64]) {
        self.width = self.width.min(vec.len());
        let num_chunks = vec.len() / self.width;
        let iter = vec.chunks(num_chunks);
        for x in iter {
            let sum: f64 = x.iter().sum();
            self.x_axis.push(sum / x.len() as f64);
        }
        let step = (self.stats.max - self.stats.min) / self.height as f64;
        for y in 0..self.height {
            self.y_axis.push(self.stats.min + step * y as f64);
        }
    }
}

impl fmt::Display for XyPlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stats)?;
        let _step = (self.stats.max - self.stats.min) / self.height as f64;
        let f64fmt = match self.precision {
            None => F64Formatter::new_with_range(self.stats.min..self.stats.max),
            Some(n) => F64Formatter::new(n),
        };
        let y_width = self
            .y_axis
            .iter()
            .map(|v| f64fmt.format(*v).len())
            .max()
            .unwrap();
        let mut newvec = self.y_axis.to_vec();
        newvec.reverse();
        print_line(f, &self.x_axis, newvec[0]..f64::INFINITY, y_width, &f64fmt)?;
        for y in newvec.windows(2) {
            print_line(f, &self.x_axis, y[1]..y[0], y_width, &f64fmt)?;
        }
        Ok(())
    }
}

fn print_line(
    f: &mut fmt::Formatter,
    x_axis: &[f64],
    range: Range<f64>,
    y_width: usize,
    f64fmt: &F64Formatter,
) -> fmt::Result {
    let mut row = format!("{: <width$}", "", width = x_axis.len());
    // The reverse in the enumeration is to avoid breaking char boundaries
    // because of unicode char ● having more bytes than ascii chars.
    for (x, value) in x_axis.iter().enumerate().rev() {
        if range.contains(value) {
            row.replace_range(x..x + 1, "●".as_ref());
        }
    }
    writeln!(
        f,
        "[{}] {}",
        Blue.paint(format!(
            "{:>width$}",
            f64fmt.format(range.start),
            width = y_width
        )),
        Red.paint(row),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;
    use yansi::Paint;

    #[test]
    fn basic_test() {
        let stats = Stats::new(&[-1.0, 4.0], None);
        let mut plot = XyPlot::new(3, 5, stats, Some(3));
        plot.load(&[-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, -1.0]);
        assert_float_eq!(plot.x_axis[0], -0.5, rmax <= f64::EPSILON);
        assert_float_eq!(plot.x_axis[1], 1.5, rmax <= f64::EPSILON);
        assert_float_eq!(plot.x_axis[2], 3.5, rmax <= f64::EPSILON);
        assert_float_eq!(plot.x_axis[3], -1.0, rmax <= f64::EPSILON);

        assert_float_eq!(plot.y_axis[0], -1.0, rmax <= f64::EPSILON);
        assert_float_eq!(plot.y_axis[4], 3.0, rmax <= f64::EPSILON);
    }

    #[test]
    fn display_test() {
        let stats = Stats::new(&[-1.0, 4.0], None);
        let mut plot = XyPlot::new(3, 5, stats, Some(3));
        plot.load(&[-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, -1.0]);
        Paint::disable();
        let display = format!("{}", plot);
        assert!(display.contains("[ 3.000]   ● "));
        assert!(display.contains("[ 2.000]     "));
        assert!(display.contains("[ 1.000]  ●  "));
        assert!(display.contains("[-1.000] ●  ●"));
    }

    #[test]
    fn display_test_human_units() {
        let vector = &[1000000.0, -1000000.0, -2000000.0, -4000000.0];
        let stats = Stats::new(vector, None);
        let mut plot = XyPlot::new(3, 5, stats, None);
        plot.load(vector);
        Paint::disable();
        let display = format!("{}", plot);
        assert!(display.contains("[    0 K] ●   "));
        assert!(display.contains("[-1000 K]  ●  "));
        assert!(display.contains("[-2000 K]   ● "));
        assert!(display.contains("[-3000 K]     "));
        assert!(display.contains("[-4000 K]    ●"));
        assert!(display.contains("Samples = 4; Min = -4000 K; Max = 1000 K"));
        assert!(display.contains("Average = -1500 K;"));
    }
}
