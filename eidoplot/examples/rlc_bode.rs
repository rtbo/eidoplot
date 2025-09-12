use std::f64::consts::PI;

use eidoplot::{eplt, data};

mod common;

fn logspace(start: f64, end: f64, num: usize) -> Vec<f64> {
    let log_start = start.log10();
    let log_end = end.log10();
    let step = (log_end - log_start) / (num as f64 - 1.0);
    (0..num)
        .map(|i| 10f64.powf(log_start + i as f64 * step))
        .collect()
}

/// Computes the transfer function of a series RLC circuit, with a load across the capacitor.
/// The input vector is the frequencies in Hz
/// The returned vectors are the magnitude in dB and the phase in radians
fn rlc_load_response(
    frequencies: &[f64],
    r: f64,
    l: f64,
    c: f64,
    r_load: f64,
) -> (Vec<f64>, Vec<f64>) {
    let mut mag = Vec::with_capacity(frequencies.len());
    let mut phase = Vec::with_capacity(frequencies.len());

    for &f in frequencies {
        let omega = 2.0 * PI * f;
        let omega_sq = omega.powi(2);

        let denom_real = r + r_load - l * r_load * c * omega_sq;
        let denom_imag = omega * (r * r_load * c + l);

        let m = r_load / (denom_real.powi(2) + denom_imag.powi(2)).sqrt();
        mag.push(20.0 * m.log10());

        // Phase in radians
        let ph = -denom_imag.atan2(denom_real);
        phase.push(ph);
    }

    (mag, phase)
}

fn main() {
    const R: f64 = 3.0;
    const L: f64 = 10e-3; // 10 mH
    const C: f64 = 1e-6; // 1 uF
    const R_LOAD: f64 = 0.0; 

    let freq = logspace(10.0, 10000.0, 500);
    let (mag, phase) = rlc_load_response(&freq, R, L, C, R_LOAD);

    let filename = common::example_res("rlc-bode.eplt");
    let content = std::fs::read_to_string(&filename).unwrap();

    let mut source = data::NamedColumns::new(); 
    source.add_column("freq", &freq as &dyn data::Column);
    source.add_column("mag", &mag as &dyn data::Column);
    source.add_column("phase", &phase as &dyn data::Column);

    let figs = eplt::parse_diag(&content, Some(&filename)).unwrap();
    common::save_figure(&figs[0], &source, "bode_rlc_gain");
    common::save_figure(&figs[1], &source, "bode_rlc_phase");
}
