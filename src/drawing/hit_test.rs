use crate::ir::PlotIdx;
use crate::{data, geom};

#[derive(Debug, Clone)]
pub struct HitCoord<'a>(data::SampleRef<'a>, &'a str);

impl HitCoord<'_> {
    pub fn as_sample(&self) -> data::SampleRef<'_> {
        self.0
    }

    pub fn as_str(&self) -> &str {
        self.1
    }
}

/// Coordinates of a data point in a plot, along with its corresponding data sample, for each axis in the plot.
/// This can be for either x or y axes.
// implementation note: we use Option<PlotCoord> when there is a single axis (most common case),
// and Vec<PlotCoord> when there are multiple axes.
// An empty plot will have None and empty Vec.
// There can't be Some and non-empty Vec at the same time.
#[derive(Debug, Clone, Default)]
pub struct PlotCoords(Option<(data::Sample, String)>, Vec<(data::Sample, String)>);

impl PlotCoords {
    pub fn len(&self) -> usize {
        if self.0.is_some() { 1 } else { self.1.len() }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<HitCoord<'_>> {
        if let Some(pc) = &self.0 {
            if index == 0 {
                return Some(HitCoord(pc.0.as_ref(), pc.1.as_str()));
            } else {
                return None;
            }
        }
        self.1
            .get(index)
            .map(|pc| HitCoord(pc.0.as_ref(), pc.1.as_str()))
    }

    fn push(&mut self, sample: (data::Sample, String)) {
        if self.0.is_none() && self.1.is_empty() {
            self.0.replace(sample);
        } else if self.0.is_some() {
            let first = self.0.take().unwrap();
            self.1.push(first);
            self.1.push(sample);
        } else {
            self.1.push(sample);
        }
    }
}

impl std::fmt::Display for PlotCoords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pc) = &self.0 {
            f.write_str(&pc.1)
        } else {
            for (i, pc) in self.1.iter().enumerate() {
                if i > 0 {
                    f.write_str("  |  ")?;
                }
                f.write_str(&pc.1)?;
            }
            Ok(())
        }
    }
}

/// Result of a hit test on a figure
#[derive(Debug, Clone)]
pub struct PlotHit {
    /// Index of the plot that was hit
    pub idx: PlotIdx,
    /// Coordinates on the x axes of the plot
    pub x_coords: PlotCoords,
    /// Coordinates on the y axes of the plot
    pub y_coords: PlotCoords,
}

impl super::Figure {
    /// Perform a hit test on the figure for the given point in figure coordinates.
    pub fn hit_test(&self, point: geom::Point) -> Option<PlotHit> {
        for p in self.plots.plots().iter().filter_map(Option::as_ref) {
            let rect = p.rect();
            if rect.contains_point(&point) {
                let point = geom::Point {
                    x: point.x - rect.x(),
                    y: rect.bottom() - point.y,
                };
                if let Some(axes) = p.axes() {
                    let x_coords = axes_coords(&axes.x(), point.x);
                    let y_coords = axes_coords(&axes.y(), point.y);
                    return Some(PlotHit {
                        idx: p.idx(),
                        x_coords,
                        y_coords,
                    });
                }
            }
        }
        None
    }

    /// Perform a hit test on the figure for the given point in figure coordinates.
    /// Only checks if a plot is hit, and returns its index.
    pub fn hit_test_idx(&self, point: geom::Point) -> Option<PlotIdx> {
        self.plots
            .plots()
            .iter()
            .filter_map(Option::as_ref)
            .find_map(|p| p.rect().contains_point(&point).then_some(p.idx()))
    }
}

fn axes_coords(axes: &[super::axis::Axis], pos: f32) -> PlotCoords {
    let mut coords = PlotCoords::default();
    for axis in axes {
        let cm = axis.coord_map();
        let sample = cm.unmap_coord(pos);
        let str = axis.format_sample(sample);
        coords.push((sample.into(), str));
    }
    coords
}
