use std::f64::consts::PI;

use plotive::{data, des, style, text, utils};

mod common;

/// Computes a single point of the transfer fonction of a series RLC circuit, with output across capacitor
/// The frequency f is in Hz, r is the resistance, l is the inductance and c is the capacitance.
/// The returned values are the magnitude in dB and phase in rad at this frequency
fn rlc_freq_response(f: f64, r: f64, l: f64, c: f64) -> (f64, f64) {
    let pulse = 2.0 * PI * f;

    // H(jw) = 1 / (1 - w^2LC + jwRC)

    let num = 1.0;
    let real = 1.0 - pulse * pulse * l * c;
    let imag = pulse * r * c;

    let mag = num / (real.powi(2) + imag.powi(2)).sqrt();
    let ph = -(imag / real).atan();
    (20.0 * mag.log10(), ph)
}

/// Computes the transfer function of a series RLC circuit, with output across the capacitor.
/// The input vector is the frequencies in Hz
/// The returned vectors are the magnitude in dB and the phase in radians
fn rlc_full_response(frequencies: &[f64], r: f64, l: f64, c: f64) -> (Vec<f64>, Vec<f64>) {
    let mut mags = Vec::with_capacity(frequencies.len());
    let mut phases = Vec::with_capacity(frequencies.len());

    for &f in frequencies {
        let (mag, phase) = rlc_freq_response(f, r, l, c);

        mags.push(mag);
        phases.push(phase);
    }

    (mags, phases)
}

fn lc_cutoff_freq(l: f64, c: f64) -> f64 {
    1.0 / (2.0 * PI * (l * c).sqrt())
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
    let mag_freq_axis = des::Axis::new()
        .with_scale(des::axis::ref_id("Frequency [Hz]").into())
        .with_ticks(Default::default())
        .with_minor_ticks(Default::default());
    let mag_axis = des::Axis::new()
        .with_title("Magnitude [dB]".into())
        .with_ticks(Default::default())
        .with_grid(Default::default());

    let phase_freq_axis = des::Axis::new()
        .with_title("Frequency [Hz]".into())
        .with_scale(des::axis::LogScale::default().into())
        .with_ticks(Default::default())
        .with_minor_ticks(Default::default());
    let phase_axis = des::Axis::new()
        .with_title("Phase [rad]".into())
        .with_ticks(
            des::axis::Ticks::new()
                .with_locator(des::axis::ticks::PiMultipleLocator::default().into()),
        )
        .with_grid(Default::default());

    let mut mag_series: Vec<des::Series> = Vec::with_capacity(3);
    let mut phase_series: Vec<des::Series> = Vec::with_capacity(3);

    let mut source = data::NamedOwnedColumns::new();

    let freq = utils::logspace(100.0, 1000000.0, 500);

    for (r, mag_col, phase_col, name) in series {
        let (mag, phase) = rlc_full_response(&freq, r, L, C);

        source.add_column(mag_col, Box::new(mag));
        source.add_column(phase_col, Box::new(phase));

        // name only on the magnitude to avoid double legend
        mag_series.push(
            des::series::Line::new(des::data_src_ref("freq"), des::data_src_ref(mag_col))
                .with_name(name)
                .into(),
        );
        phase_series.push(
            des::series::Line::new(des::data_src_ref("freq"), des::data_src_ref(phase_col)).into(),
        );
    }

    source.add_column("freq", Box::new(freq));

    // cut-off frequency
    let cutoff = lc_cutoff_freq(L, C);
    // magnitude two decades after cut-off (to increase precision)
    let mag_2_decades = rlc_freq_response(cutoff * 100.0, 1.0, L, C).0;

    let cutoff_line =
        des::annot::Line::vertical(cutoff).with_pattern(style::Dash::default().into());
    let slope_line = des::annot::Line::two_points(cutoff, 0.0, 100.0 * cutoff, mag_2_decades)
        .with_pattern(style::Dash::default().into());
    let cut_off_label =
        des::annot::Label::new(format!("{:.2} kHz", cutoff / 1000.0), cutoff, -60.0)
            .with_anchor(des::annot::Anchor::BottomLeft)
            .with_angle(90.0);
    let slope_label = des::annot::Label::new(
        format!("{:.0} dB/decade", mag_2_decades / 2.0),
        cutoff * 10.0,
        mag_2_decades / 2.0,
    )
    .with_anchor(des::annot::Anchor::BottomLeft);

    let mag_plot = des::Plot::new(mag_series)
        .with_x_axis(mag_freq_axis)
        .with_y_axis(mag_axis)
        .with_annotation(cutoff_line.into())
        .with_annotation(slope_line.into())
        .with_annotation(cut_off_label.into())
        .with_annotation(slope_label.into());

    let phase_plot = des::Plot::new(phase_series)
        .with_x_axis(phase_freq_axis)
        .with_y_axis(phase_axis);

    let fig = des::Figure::new(
        des::Subplots::new(2, 1)
            .with_plot((0, 0), mag_plot)
            .with_plot((1, 0), phase_plot)
            .into(),
    )
    .with_title(title.into())
    .with_legend(des::figure::LegendPos::Right.into());

    common::save_figure(&fig, &source, None, "bode_rlc");
}
