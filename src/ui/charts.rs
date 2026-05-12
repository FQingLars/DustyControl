use std::collections::VecDeque;

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke};
use iced::{Color, Element, Point, Rectangle, Renderer, Theme};

const CHART_GRID: Color = Color::from_rgb(0.13, 0.17, 0.23);

pub struct MultiLineChart {
    pub data: Vec<(VecDeque<f64>, Color)>,
    pub label: String,
    pub max_value: f64,
}

impl MultiLineChart {
    pub fn new(label: &str, max_value: f64) -> Self {
        Self {
            data: Vec::new(),
            label: label.to_string(),
            max_value,
        }
    }

    pub fn set_data(&mut self, data: Vec<(VecDeque<f64>, Color)>) {
        self.data = data;
    }

    pub fn update(&mut self) {}

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        Canvas::new(ChartProgram {
            data: self.data.clone(),
            max_value: self.max_value,
        })
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(140.0))
        .into()
    }
}

#[derive(Clone)]
pub struct ChartProgram {
    data: Vec<(VecDeque<f64>, Color)>,
    max_value: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {}

impl canvas::Program<Message> for ChartProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = frame.width();
        let h = frame.height();
        let padding = 15.0;
        let plot_w = w - padding * 2.0;
        let plot_h = h - padding * 2.0;

        if let Some(first) = self.data.first() {
            let len = first.0.len();
            if len >= 2 {
                for (series, color) in &self.data {
                    if series.len() < 2 {
                        continue;
                    }

                    let max_val: f64 = self
                        .max_value
                        .max(series.iter().fold(0.0f64, |a, &b| a.max(b)) * 1.1);
                    if max_val <= 0.0 {
                        continue;
                    }

                    let path = Path::new(|p| {
                        let mut first_point = true;
                        for (i, val) in series.iter().enumerate() {
                            let x = padding
                                + (i as f32 / (len - 1) as f32) * plot_w as f32;
                            let y = padding
                                + (1.0 - (*val as f32 / max_val as f32)) * plot_h as f32;
                            if first_point {
                                p.move_to(Point::new(x, y));
                                first_point = false;
                            } else {
                                p.line_to(Point::new(x, y));
                            }
                        }
                    });

                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_color(*color)
                            .with_width(2.0),
                    );
                }
            }
        }

        for i in 0..4 {
            let y = padding + (i as f32 / 3.0) * plot_h as f32;
            let grid_line = Path::line(
                Point::new(padding, y),
                Point::new(padding + plot_w, y),
            );
            frame.stroke(
                &grid_line,
                Stroke::default()
                    .with_color(CHART_GRID)
                    .with_width(1.0),
            );
        }

        vec![frame.into_geometry()]
    }
}
