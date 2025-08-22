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
pub use figure::Figure;
pub use legend::Legend;
pub use plot::Plot;
pub use series::{Series, SeriesPlot};
