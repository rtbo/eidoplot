/*!
 * # Intermediate representation (IR) for eidoplot
 * 
 * This module contains all data structures for the design of plotting figures.
 */
pub mod axis;
pub mod figure;
pub mod legend;
pub mod plot;
pub mod series;

pub use axis::Axis;
pub use figure::{Figure, FigLegend};
pub use legend::Legend;
pub use plot::{Plot, PlotLegend};
pub use series::{Series, SeriesPlot};
