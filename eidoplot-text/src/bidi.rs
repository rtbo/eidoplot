
#[derive(Debug)]
pub struct BidiRun {
    pub start: usize,
    pub end: usize,
    pub dir: rustybuzz::Direction,
}

#[derive(Debug)]
pub enum BidiAlgo {
    Nope(rustybuzz::Direction),
    Yep {
        default_lev: Option<unicode_bidi::Level>,
    },
}

impl BidiAlgo {
    pub fn start_dir(&self) -> rustybuzz::Direction {
        match self {
            BidiAlgo::Nope(dir) => *dir,
            BidiAlgo::Yep { default_lev } => {
                if let Some(lev) = default_lev {
                    if lev.is_rtl() {
                        rustybuzz::Direction::RightToLeft
                    } else {
                        rustybuzz::Direction::LeftToRight
                    }
                } else {
                    rustybuzz::Direction::LeftToRight
                }
            }
        }
    }

    pub fn visual_runs(&mut self, text: &str, start: usize) -> Vec<BidiRun> {
        match self {
            BidiAlgo::Nope(dir) => vec![BidiRun {
                start,
                end: start + text.len(),
                dir: *dir,
            }],
            BidiAlgo::Yep { default_lev } => {
                let bidi = unicode_bidi::BidiInfo::new(text, *default_lev);
                let mut res_runs = Vec::new();

                for para in &bidi.paragraphs {
                    let line = para.range.clone();
                    let (levels, runs) = bidi.visual_runs(para, line);
                    for run in runs {
                        let lev = levels[run.start];
                        let dir = if lev.is_rtl() {
                            rustybuzz::Direction::RightToLeft
                        } else {
                            rustybuzz::Direction::LeftToRight
                        };
                        if default_lev.is_none() {
                            // assign for following lines
                            *default_lev = Some(lev);
                        }
                        res_runs.push(BidiRun {
                            start: start + run.start,
                            end: start + run.end,
                            dir,
                        })
                    }
                }

                if res_runs.is_empty() {
                    let dir = if let Some(lev) = default_lev {
                        if lev.is_rtl() {
                            rustybuzz::Direction::RightToLeft
                        } else {
                            rustybuzz::Direction::LeftToRight
                        }
                    } else {
                        rustybuzz::Direction::LeftToRight
                    };
                    res_runs.push(BidiRun {
                        start,
                        end: start + text.len(),
                        dir,
                    });
                }
                res_runs
            }
        }
    }
}
