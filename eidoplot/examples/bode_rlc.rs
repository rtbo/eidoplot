use std::f64::consts::PI;

use eidoplot::{data, ir, style};
use eidoplot_text as text;

mod common;

/// Computes the transfer function of a series RLC circuit, with output across the capacitor.
/// The input vector is the frequencies in Hz
/// The returned vectors are the magnitude in dB and the phase in radians
fn rlc_load_response(frequencies: &[f64], r: f64, l: f64, c: f64) -> (Vec<f64>, Vec<f64>) {
    let mut mags = Vec::with_capacity(frequencies.len());
    let mut phases = Vec::with_capacity(frequencies.len());

    for &f in frequencies {
        let pulse = 2.0 * PI * f;

        let num = 1.0;
        let denom_real = 1.0 - pulse * pulse * l * c;
        let denom_imag = pulse * r * c;

        let mag = num / (denom_real.powi(2) + denom_imag.powi(2)).sqrt();
        let ph = -(denom_imag / denom_real).atan();

        mags.push(20.0 * mag.log10());
        phases.push(ph);
    }

    (mags, phases)
}

fn main() {
    const L: f64 = 1e-4; // 100 µH
    const C: f64 = 1e-6; // 1 uF

    let series = [
        (1.0, "mag1", "phase1", "R = 1 Ω"),
        (10.0, "mag2", "phase2", "R = 10 Ω"),
        (100.0, "mag3", "phase3", "R = 100 Ω"),
    ];

    let title = text::parse_rich_text::<style::theme::Color>(concat!(
        "Bode diagram of RLC circuit\n",
        "[size=18;italic;font=serif]L = 0.1 mH / C = 1 µF[/size;italic;font]"
    ))
    .unwrap();

    // magnitude X axis scale is taken from the phase X axis
    // the reference uses the title given to the phase X axis
    let mag_freq_axis = ir::Axis::new()
        .with_scale(ir::axis::Scale::Shared(ir::axis::Ref::Title(
            "Frequency [Hz]".to_string(),
        )))
        .with_ticks(Default::default())
        .with_minor_ticks(Default::default());
    let mag_axis = ir::Axis::new()
        .with_title("Magnitude [dB]".to_string().into())
        .with_ticks(Default::default())
        .with_grid(Default::default());

    let phase_freq_axis = ir::Axis::new()
        .with_title("Frequency [Hz]".to_string().into())
        .with_scale(ir::axis::LogScale::default().into())
        .with_ticks(Default::default())
        .with_minor_ticks(Default::default());
    let phase_axis = ir::Axis::new()
        .with_title("Phase [rad]".to_string().into())
        .with_ticks(
            ir::axis::Ticks::new().with_locator(ir::axis::ticks::Locator::PiMultiple { bins: 9 }),
        )
        .with_grid(Default::default());

    let mut mag_series: Vec<ir::Series> = Vec::with_capacity(3);
    let mut phase_series: Vec<ir::Series> = Vec::with_capacity(3);

    let mut source = data::NamedOwnedColumns::new();

    let freq = common::logspace(100.0, 1000000.0, 500);

    for (r, mag_col, phase_col, name) in series {
        let (mag, phase) = rlc_load_response(&freq, r, L, C);

        source.add_column(mag_col, Box::new(mag));
        source.add_column(phase_col, Box::new(phase));

        // name only on the magnitude to avoid double legend
        mag_series.push(
            ir::series::Line::new(
                ir::DataCol::SrcRef("freq".to_string()),
                ir::DataCol::SrcRef(mag_col.to_string()),
            )
            .with_name(name.to_string())
            .into(),
        );
        phase_series.push(
            ir::series::Line::new(
                ir::DataCol::SrcRef("freq".to_string()),
                ir::DataCol::SrcRef(phase_col.to_string()),
            )
            .into(),
        );
    }

    source.add_column("freq", Box::new(freq));

    let mag_plot = ir::Plot::new(mag_series)
        .with_x_axis(mag_freq_axis)
        .with_y_axis(mag_axis);
    let phase_plot = ir::Plot::new(phase_series)
        .with_x_axis(phase_freq_axis)
        .with_y_axis(phase_axis);

    let fig = ir::Figure::new(
        ir::Subplots::new(2, 1)
            .with_plot(0, 0, mag_plot)
            .with_plot(0, 1, phase_plot)
            .with_space(10.0)
            .into(),
    )
    .with_title(title.into())
    .with_legend(ir::figure::LegendPos::Right.into());

    common::save_figure(&fig, &source, "bode_rlc");
}
