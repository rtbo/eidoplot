use plotive::des;

mod common;

const START_HEIGHT: f64 = 60.0;
const BOUNCE_RESTITUTION: f64 = 0.76;

type Time = f64;
type State = ode_solvers::Vector2<f64>; // stores height and velocity
const H: usize = 0;
const V: usize = 1;

struct BouncingBall;

impl ode_solvers::System<Time, State> for BouncingBall {
    fn system(&self, _t: Time, y: &State, dy: &mut State) {
        // approximate parameters for a tennis ball
        // drag coefficient
        let cd = 0.5;
        // ball mass
        let m = 0.058;
        // ball diameter
        let d = 0.065;
        // frontal surface
        let s = core::f64::consts::PI * d * d / 4.0;
        // air density
        let rho = 1.225;
        // gravity constant
        let g = 9.81;

        let vel = y[1];
        let drag_force = 0.5 * rho * vel * vel * s * cd;

        let drag_direction = if vel >= 0.0 { 1.0 } else { -1.0 };

        dy[H] = y[V];
        dy[V] = -g - drag_direction * drag_force / m;
    }

    fn solout(&mut self, _x: Time, y: &State, _dy: &State) -> bool {
        // stop when the ball hits the ground
        y[H] < 0.
    }
}

struct BouncingBallData {
    time: Vec<f64>,
    location: Vec<f64>,
    velocity: Vec<f64>,
}

impl BouncingBallData {
    fn calculate() -> Self {
        let mut res = BouncingBallData {
            time: Vec::new(),
            location: Vec::new(),
            velocity: Vec::new(),
        };

        let mut x0 = 0.0;
        let mut y0 = State::new(START_HEIGHT, 0.0);
        let x_end = 10.0;

        let mut bounce = 1;
        let max_bounce = 10;

        loop {
            let bb = BouncingBall;
            let mut stepper = ode_solvers::Dopri5::new(bb, x0, x_end, 0.01, y0, 1E-6, 1E-6);
            let stats = stepper.integrate().unwrap();
            println!("Bounce {bounce}\n{stats}\n");
            bounce += 1;

            let (x, y) = stepper.results().get();
            let rebounce = y
                .iter()
                .enumerate()
                .find(|(_i, y)| y[H] <= 0.0)
                .map(|(i, _y)| i);

            let (x, y) = if let Some(touch_ground) = rebounce {
                (&x[..=touch_ground], &y[..=touch_ground])
            } else {
                (x.as_slice(), y.as_slice())
            };

            res.time.extend(x.iter());
            res.location.extend(y.iter().map(|y| y[0]));
            res.velocity.extend(y.iter().map(|y| y[1]));

            let Some(rebounce) = rebounce else { break };
            let x = &x[..=rebounce];
            let y = &y[..=rebounce];

            x0 = *x.last().unwrap();
            y0 = *y.last().unwrap();
            y0[H] = y0[H].abs();
            y0[V] *= -BOUNCE_RESTITUTION;

            if x0 >= x_end || bounce >= max_bounce {
                break;
            }
        }

        res
    }
}

fn main() {
    let data = BouncingBallData::calculate();
    let mut data_source = plotive::data::NamedColumns::new();
    data_source.add_column("time", &data.time);
    data_source.add_column("height", &data.location);
    data_source.add_column("velocity", &data.velocity);

    let title = "Tennis ball thrown from 1\u{02E2}\u{1D57} floor of Eiffel Tower";

    let fig = des::Figure::new(
        des::Plot::new(vec![
            des::series::Line::new(des::data_src_ref("time"), des::data_src_ref("height"))
                .with_name("Height (m)")
                .into(),
            des::series::Line::new(des::data_src_ref("time"), des::data_src_ref("velocity"))
                .with_name("Velocity (m/s)")
                .into(),
        ])
        .with_x_axis(
            des::Axis::new()
                .with_ticks(
                    des::axis::Ticks::new()
                        .with_locator(des::axis::ticks::TimeDeltaLocator::Seconds(2).into())
                        .with_formatter(Some(
                            des::axis::ticks::TimeDeltaFormatter::Custom("%M:%S".to_string())
                                .into(),
                        )),
                )
                .with_grid(Default::default())
                .with_title("Time".into()),
        )
        .with_y_axis(
            des::Axis::new()
                .with_ticks(des::axis::Ticks::new())
                .with_grid(Default::default()),
        )
        .with_annotation(des::annot::Line::horizontal(0.0).into())
        .with_legend(des::plot::LegendPos::InTopRight.into())
        .into(),
    )
    .with_title(title.into());

    common::save_figure(&fig, &data_source, None, "bouncing_ball");
}
