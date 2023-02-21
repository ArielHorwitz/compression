use plotly::{
    common::{Mode, Title},
    layout::Axis,
    Layout, Plot, Scatter,
};

pub fn plot(
    data: Vec<f64>,
    resolution: f64,
    title: &str,
    file_path: &str,
    x_axis: &str,
    y_axis: &str,
) {
    let freq_legend = (0..data.len()).map(|x| x as f64 * resolution).collect();
    let mut plot = Plot::new();
    let trace = Scatter::new(freq_legend, data.clone()).mode(Mode::Lines);
    plot.add_trace(trace);
    let layout = Layout::new()
        .title(Title::new(title))
        .x_axis(Axis::new().title(Title::from(x_axis)))
        .y_axis(Axis::new().title(Title::from(y_axis)))
        .width(1900)
        .height(800);
    plot.set_layout(layout);
    plot.write_html(file_path);
}
