use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eidoplot::data::Column;
use eidoplot::drawing::FigureDraw;
use eidoplot::{data, drawing, geom, ir};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream};
use iced::widget::{button, column, container, pick_list, row, space, text};
use iced::{Alignment, Element, Subscription, Task, event, window};
use iced::{Length, stream};
use iced_oplot::figure;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex32;

#[derive(Debug, Clone)]
enum Message {
    SelectHost(&'static str),
    SelectDevice(String),
    SelectBufferSize(u32),
    SelectSampleRate(u32),
    StartStop,
    AudioLoop(mpsc::Sender<AudioLoopMsg>),
    AudioData(AudioBuffer),
    WindowCloseRequested,
}

#[derive(Clone)]
enum AudioLoopMsg {
    Start {
        device: cpal::Device,
        buffer_size: u32,
        sample_rate: u32,
    },
    Stop,
    Exit,
}

#[derive(Debug, Clone)]
struct AudioBuffer {
    buf: Arc<Mutex<Vec<f32>>>,
}

impl AudioBuffer {
    fn new() -> Self {
        Self {
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_data<F>(&self, f: F)
    where
        F: FnOnce(&[f32]),
    {
        let src = self.buf.lock().unwrap();
        f(&src);
    }

    fn set_data(&self, data: &[f32]) {
        let mut dest = self.buf.lock().unwrap();
        dest.resize(data.len(), 0.0);
        dest.copy_from_slice(data);
    }
}

/// A one second rolling buffer for audio samples.
#[derive(Debug)]
struct AudioScopeDataSrc {
    time_col: TimeColumn,
    time_y_col: RollingAudioBuffer,
    freq_col: FreqColumn,
    freq_y_col: FftColumn,
}

impl AudioScopeDataSrc {
    fn new(sample_rate: usize, buffer_size: usize) -> Self {
        Self {
            time_col: TimeColumn {
                sample_rate: sample_rate as f32,
                len: buffer_size,
            },
            time_y_col: RollingAudioBuffer::new(buffer_size),
            freq_col: FreqColumn {
                sample_rate: sample_rate as f32,
                len: buffer_size,
            },
            freq_y_col: FftColumn::new(buffer_size),
        }
    }

    fn add_samples(&mut self, samples: &[f32]) {
        if samples.len() != self.time_col.len {
            self.time_col.resize(samples.len());
            self.time_y_col.resize(samples.len());
            self.freq_col.resize(samples.len());
            self.freq_y_col.resize(samples.len());
        }
        self.time_y_col.add_samples(samples);
        self.freq_y_col.add_samples(samples);
    }
}

fn hanning(i: usize, n: usize) -> f32 {
    0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos())
}

#[derive(Debug)]
struct TimeColumn {
    sample_rate: f32,
    len: usize,
}

impl TimeColumn {
    fn resize(&mut self, new_len: usize) {
        self.len = new_len;
    }
}

impl data::F64Column for TimeColumn {
    fn len(&self) -> usize {
        self.len
    }

    fn len_some(&self) -> usize {
        self.len
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new((0..self.len).map(move |i| Some(i as f64 / self.sample_rate as f64)))
    }
}

impl data::Column for TimeColumn {
    fn f64(&self) -> Option<&dyn data::F64Column> {
        Some(self)
    }

    fn len(&self) -> usize {
        self.len
    }

    fn len_some(&self) -> usize {
        self.len
    }
}

#[derive(Debug)]
struct RollingAudioBuffer {
    samples: Vec<f32>,
    cursor: usize,
}

impl RollingAudioBuffer {
    fn new(data_len: usize) -> Self {
        Self {
            samples: vec![0.0; data_len],
            cursor: 0,
        }
    }

    fn resize(&mut self, new_len: usize) {
        self.samples.resize(new_len, 0.0);
        if self.cursor >= new_len {
            self.cursor = 0;
        }
    }

    fn add_samples(&mut self, mut samples: &[f32]) {
        // drain samples in time_y
        while !samples.is_empty() {
            let space_left = self.samples.len() - self.cursor;
            let to_copy = space_left.min(samples.len());
            self.samples[self.cursor..self.cursor + to_copy].copy_from_slice(&samples[..to_copy]);
            self.cursor += to_copy;
            if self.cursor >= self.samples.len() {
                self.cursor = 0;
            }
            samples = &samples[to_copy..];
        }
    }
}

impl data::Column for RollingAudioBuffer {
    fn f64(&self) -> Option<&dyn data::F64Column> {
        Some(self)
    }

    fn len(&self) -> usize {
        self.samples.len()
    }

    fn len_some(&self) -> usize {
        self.samples.len()
    }
}

impl data::F64Column for RollingAudioBuffer {
    fn len(&self) -> usize {
        self.samples.len()
    }

    fn len_some(&self) -> usize {
        self.samples.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        let len = self.samples.len();
        let cursor = self.cursor;
        Box::new((0..len).map(move |i| {
            let idx = (cursor + i) % len;
            Some(self.samples[idx] as f64)
        }))
    }
}

#[derive(Debug)]
struct FreqColumn {
    sample_rate: f32,
    len: usize,
}

impl FreqColumn {
    fn resize(&mut self, new_len: usize) {
        self.len = new_len;
    }
}

impl data::F64Column for FreqColumn {
    fn len(&self) -> usize {
        self.len / 2
    }

    fn len_some(&self) -> usize {
        self.len / 2
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        Box::new(
            (0..self.len / 2)
                .map(move |i| Some(i as f64 * self.sample_rate as f64 / self.len as f64)),
        )
    }
}

impl Column for FreqColumn {
    fn f64(&self) -> Option<&dyn data::F64Column> {
        Some(self)
    }

    fn len(&self) -> usize {
        self.len / 2
    }

    fn len_some(&self) -> usize {
        self.len / 2
    }
}

struct FftColumn {
    plan: Arc<dyn rustfft::Fft<f32> + Send + Sync>,
    data: Vec<Complex32>,
    scratch: Vec<Complex32>,
}

impl std::fmt::Debug for FftColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FftColumn")
            .field("len", &self.data.len())
            .finish()
    }
}

impl FftColumn {
    fn new(size: usize) -> Self {
        let mut fft_planner = FftPlanner::new();
        let plan = fft_planner.plan_fft_forward(size);
        let data = vec![Complex32::new(0.0, 0.0); size];
        let scratch = vec![Complex32::new(0.0, 0.0); size];
        Self {
            plan,
            data,
            scratch,
        }
    }

    fn resize(&mut self, new_len: usize) {
        self.data.resize(new_len, Complex32::new(0.0, 0.0));
        self.scratch.resize(new_len, Complex32::new(0.0, 0.0));
        let mut fft_planner = FftPlanner::new();
        self.plan = fft_planner.plan_fft_forward(new_len);
    }

    fn add_samples(&mut self, samples: &[f32]) {
        // copy samples to fft buffer with hanning windowing
        for i in 0..self.data.len() {
            self.data[i].re = hanning(i, self.data.len()) * samples[i];
            self.data[i].im = 0.0;
        }
        // perform fft
        self.plan
            .process_with_scratch(&mut self.data, &mut self.scratch);
    }
}

impl data::Column for FftColumn {
    fn f64(&self) -> Option<&dyn data::F64Column> {
        Some(self)
    }

    fn len(&self) -> usize {
        self.data.len() / 2
    }

    fn len_some(&self) -> usize {
        self.data.len() / 2
    }
}

impl data::F64Column for FftColumn {
    fn len(&self) -> usize {
        self.data.len() / 2
    }

    fn len_some(&self) -> usize {
        self.data.len() / 2
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Option<f64>> + '_> {
        let len = self.data.len();
        // Scale: amplitude -> dBFS (reference 1.0), normalized by N and window gain.
        // rustfft does not normalize: raw magnitude grows with N.
        let n = len as f32;
        let coherent_gain_hann = 0.5_f32; // coherent gain of the Hann window for amplitude
        let eps = 1e-12_f32; // avoid -inf
        Box::new((0..len / 2).map(move |i| {
            let mag = self.data[i].norm() / (n * coherent_gain_hann);
            let db = 20.0_f32 * (mag.max(eps)).log10();
            Some(db as f64)
        }))
    }
}

impl data::Source for AudioScopeDataSrc {
    fn column(&self, name: &str) -> Option<&dyn data::Column> {
        match name {
            "time" => Some(&self.time_col),
            "time_y" => Some(&self.time_y_col),
            "freq" => Some(&self.freq_col),
            "freq_y" => Some(&self.freq_y_col),
            _ => None,
        }
    }
}

fn build_figure() -> ir::Figure {
    ir::Figure::new(
        ir::Subplots::new(2, 1)
            .with_plot(
                0,
                0,
                ir::Plot::new(vec![
                    ir::series::Line::new(
                        ir::DataCol::SrcRef("time".to_string()),
                        ir::DataCol::SrcRef("time_y".to_string()),
                    )
                    .into(),
                ])
                .with_line(ir::PlotLine::horizontal(0.0))
                .with_x_axis(
                    ir::Axis::new()
                        .with_title("Time (s)".to_string().into())
                        .with_ticks(Default::default())
                        .with_grid(Default::default()),
                )
                .with_y_axis(
                    ir::Axis::new()
                        .with_scale(ir::axis::Range::MinMax(-1.0, 1.0).into())
                        .with_ticks(Default::default())
                        .with_grid(Default::default()),
                ),
            )
            .with_plot(
                1,
                0,
                ir::Plot::new(vec![
                    ir::series::Line::new(
                        ir::DataCol::SrcRef("freq".to_string()),
                        ir::DataCol::SrcRef("freq_y".to_string()),
                    )
                    .into(),
                ])
                .with_x_axis(
                    ir::Axis::new()
                        .with_title("Frequency (Hz)".to_string().into())
                        .with_scale(ir::axis::Range::MinMax(0.0, 4000.0).into())
                        .with_ticks(
                            ir::axis::Ticks::new()
                                .with_formatter(ir::axis::ticks::Formatter::Prec(0).into()),
                        )
                        .with_grid(Default::default()),
                )
                .with_y_axis(
                    ir::Axis::new()
                        .with_title("Amplitude (dBFS)".to_string().into())
                        .with_scale(ir::axis::Range::MinMax(-100.0, 0.0).into())
                        .with_ticks(
                            ir::axis::Ticks::new()
                                .with_formatter(ir::axis::ticks::Formatter::Prec(0).into()),
                        )
                        .with_grid(Default::default()),
                ),
            )
            .with_space(15.0)
            .into(),
    )
    .with_size(geom::Size::new(700.0, 800.0))
}

struct AudioScope {
    hosts: Vec<cpal::HostId>,
    host: Option<Arc<cpal::Host>>,
    devices: Vec<cpal::Device>,
    device: Option<cpal::Device>,
    buffer_size: Option<u32>,
    sample_rate: Option<u32>,
    rolling_buffer: AudioScopeDataSrc,
    capturing: bool,
    sender: Option<mpsc::Sender<AudioLoopMsg>>,

    fig: Option<drawing::Figure>,
}

const BUFFER_SIZES: &[u32] = &[256, 512, 1024, 2048];
const DEFAULT_BUFFER_SIZE: usize = 1024;
const SAMPLE_RATES: &[u32] = &[44100, 48000, 88200, 96000];
const DEFAULT_SAMPLE_RATE: usize = 48000;

impl Default for AudioScope {
    fn default() -> Self {
        let mut scope = AudioScope {
            hosts: cpal::available_hosts(),
            host: None,
            devices: Vec::new(),
            device: None,
            buffer_size: Some(DEFAULT_BUFFER_SIZE as u32),
            sample_rate: Some(DEFAULT_SAMPLE_RATE as u32),
            rolling_buffer: AudioScopeDataSrc::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE),
            capturing: false,
            sender: None,
            fig: None,
        };
        if scope.hosts.len() == 1 {
            let _ = scope.update(Message::SelectHost(scope.hosts[0].name()));
        }
        scope
    }
}

impl AudioScope {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectHost(host_id) => {
                self.host = self
                    .hosts
                    .iter()
                    .find(|hid| hid.name() == host_id)
                    .and_then(|hid| cpal::host_from_id(*hid).ok())
                    .map(Arc::new);
                self.devices = Vec::new();
                self.device = None;
                if let Some(host) = &self.host {
                    self.devices = host.input_devices().unwrap().collect();
                    let def_dev = host.default_input_device();
                    if let Some(def_dev) = def_dev {
                        if let Ok(name) = def_dev.name() {
                            for dev in &self.devices {
                                if dev.name().as_ref() == Ok(&name) {
                                    self.device = Some(def_dev);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Message::SelectDevice(name) => {
                self.device = self
                    .devices
                    .iter()
                    .find(|dev| dev.name().ok().as_ref() == Some(&name))
                    .map(cpal::Device::clone);
                println!(
                    "Selected device: {:?}",
                    self.device.as_ref().map(|d| d.name())
                );
            }
            Message::SelectBufferSize(bs) => {
                self.buffer_size = Some(bs);
                self.rolling_buffer = AudioScopeDataSrc::new(
                    self.sample_rate
                        .map(|sr| sr as usize)
                        .unwrap_or(DEFAULT_SAMPLE_RATE),
                    bs as usize,
                );
            }
            Message::SelectSampleRate(sr) => {
                self.sample_rate = Some(sr);
                self.rolling_buffer = AudioScopeDataSrc::new(
                    sr as usize,
                    self.buffer_size
                        .map(|bs| bs as usize)
                        .unwrap_or(DEFAULT_BUFFER_SIZE),
                );
            }
            Message::StartStop => {
                if self.capturing {
                    if let Some(sender) = &self.sender {
                        let _ = sender.clone().try_send(AudioLoopMsg::Stop);
                    }
                    self.capturing = false;
                } else if let (Some(device), Some(buffer_size), Some(sample_rate)) =
                    (&self.device, self.buffer_size, self.sample_rate)
                {
                    if let Some(sender) = &self.sender {
                        let _ = sender.clone().try_send(AudioLoopMsg::Start {
                            device: device.clone(),
                            buffer_size,
                            sample_rate,
                        });
                        self.capturing = true;
                    }
                }
            }
            Message::AudioLoop(sender) => {
                self.sender = Some(sender);
            }
            Message::AudioData(buffer) => {
                buffer.with_data(|data| {
                    self.rolling_buffer.add_samples(data);
                });
                if let Some(fig) = &mut self.fig {
                    fig.update_series_data(&self.rolling_buffer).unwrap();
                } else {
                    let fig = build_figure().prepare(&self.rolling_buffer, None).unwrap();
                    self.fig = Some(fig);
                }
            }
            Message::WindowCloseRequested => {
                if let Some(sender) = &self.sender {
                    let _ = sender.clone().try_send(AudioLoopMsg::Exit);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        const LBL_WIDTH: u32 = 120;
        const INPUT_WIDTH: u32 = 200;
        let hosts: Vec<_> = self.hosts.iter().map(|hid| hid.name()).collect();
        let hosts = pick_list(
            hosts,
            self.host.as_ref().map(|h| h.id().name()),
            Message::SelectHost,
        )
        .width(INPUT_WIDTH);
        let host_row = row![
            text("Select host:")
                .width(LBL_WIDTH)
                .align_x(Alignment::End),
            hosts,
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .padding(10);

        let devices: Vec<String> = self
            .devices
            .iter()
            .map(|d| d.name().unwrap_or("Unknown".to_string()))
            .collect();
        let devices = pick_list(
            devices,
            self.device.as_ref().and_then(|dev| dev.name().ok()),
            Message::SelectDevice,
        )
        .width(INPUT_WIDTH);
        let device_row = row![
            text("Select device:")
                .width(LBL_WIDTH)
                .align_x(Alignment::End),
            devices
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .padding(10);

        let buffer_sizes = pick_list(BUFFER_SIZES, self.buffer_size, Message::SelectBufferSize);
        let bs_row = row![
            text("Buffer Size:")
                .width(LBL_WIDTH)
                .align_x(Alignment::End),
            buffer_sizes.width(INPUT_WIDTH).placeholder("Buffer Size"),
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .padding(10);

        let sample_rates = pick_list(SAMPLE_RATES, self.sample_rate, Message::SelectSampleRate);
        let sr_row = row![
            text("Sample Rate:")
                .width(LBL_WIDTH)
                .align_x(Alignment::End),
            sample_rates.width(INPUT_WIDTH),
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .padding(10);

        let start_stop_content = if self.capturing { "Stop" } else { "Start" };
        let start_stop = text(start_stop_content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);
        let start_stop = button(start_stop)
            .width(100)
            .height(80)
            .style(|theme, status| button::primary(theme, status))
            .on_press_maybe(
                if self.device.is_some() && self.buffer_size.is_some() && self.sample_rate.is_some()
                {
                    Some(Message::StartStop)
                } else {
                    None
                },
            );

        let ctrl_col = column![
            container(start_stop).padding(10),
            space::vertical(),
            column![host_row, device_row, bs_row, sr_row,],
        ];
        let plot: Element<'_, Message> = if let Some(fig) = &self.fig {
            iced_oplot::Figure::new(&fig)
                .scale(2.0)
                .style(|theme| figure::map_theme(theme).into())
                .into()
        } else {
            text("No data")
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        };
        row![ctrl_col, plot].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(audio_subscription),
            event::listen_with(|event, _, _| match event {
                event::Event::Window(window::Event::CloseRequested) => {
                    Some(Message::WindowCloseRequested)
                }
                _ => None,
            }),
        ])
    }
}

fn audio_subscription() -> impl Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        // Create channel
        let (sender, mut receiver) = mpsc::channel(100);
        output.send(Message::AudioLoop(sender)).await.unwrap();

        let buffer = AudioBuffer::new();
        let mut stream = None;

        loop {
            use iced::futures::StreamExt;

            // Read next input sent from `Application`
            let input = receiver.select_next_some().await;
            match input {
                AudioLoopMsg::Start {
                    device,
                    buffer_size,
                    sample_rate,
                } => {
                    let config = device.default_input_config().unwrap();
                    let config = cpal::SupportedStreamConfig::new(
                        config.channels(),
                        cpal::SampleRate(sample_rate),
                        cpal::SupportedBufferSize::Range {
                            min: buffer_size,
                            max: buffer_size,
                        },
                        config.sample_format(),
                    );
                    println!("Starting capture with config {:?}...", config);
                    let buffer = buffer.clone();
                    let mut output = output.clone();
                    let strm = device
                        .build_input_stream(
                            &config.into(),
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                buffer.set_data(data);
                                if let Err(e) = output.try_send(Message::AudioData(buffer.clone()))
                                {
                                    eprintln!("Failed to send audio data: {}", e);
                                }
                            },
                            move |err| {
                                eprintln!("Stream error: {}", err);
                            },
                            None,
                        )
                        .unwrap();

                    strm.play().unwrap();
                    stream = Some(strm);
                }
                AudioLoopMsg::Stop => {
                    if let Some(strm) = stream.take() {
                        println!("Stopping capture...");
                        drop(strm);
                    }
                }
                AudioLoopMsg::Exit => {
                    break;
                }
            }
        }
    })
}

fn main() -> iced::Result {
    iced::application(AudioScope::default, AudioScope::update, AudioScope::view)
        .title("Audio Scope")
        .subscription(AudioScope::subscription)
        .run()
}
