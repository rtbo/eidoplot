use std::f64::consts::PI;

use eidoplot::{data, eplt, ir};

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

    let mut source = data::NamedOwnedColumns::new();

    let filename = common::example_res("bode-rlc.eplt");
    let content = std::fs::read_to_string(&filename).unwrap();

    let figs = eplt::parse_diag(&content, Some(&filename)).unwrap();
    let mut fig = figs.into_iter().next().unwrap();

    let freq = common::logspace(100.0, 1000000.0, 500);
    for (r, mag_col, phase_col, name) in series {
        let (mag, phase) = rlc_load_response(&freq, r, L, C);

        source.add_column(mag_col, Box::new(mag));
        source.add_column(phase_col, Box::new(phase));

        let plots = fig.plots_mut().plots_mut();
        plots[0].push_series(
            ir::series::Line::new(
                ir::DataCol::SrcRef("freq".to_string()),
                ir::DataCol::SrcRef(mag_col.to_string()),
            )
            .with_name(name.to_string())
            .into(),
        );
        plots[1].push_series(
            ir::series::Line::new(
                ir::DataCol::SrcRef("freq".to_string()),
                ir::DataCol::SrcRef(phase_col.to_string()),
            )
            .into(),
        );
    }
    source.add_column("freq", Box::new(freq));
    common::save_figure(&fig, &source, "bode_rlc");
}
