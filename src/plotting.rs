use super::common::WaveformMetadata;
use plotly::{
    common::{Mode, Title},
    layout::{Axis, GridPattern, LayoutGrid, RowOrder},
    Layout, Plot, Scatter,
};

pub fn plot(waveform: Vec<f32>, freq_bins: Vec<f32>, metadata: &WaveformMetadata, file_path: &str) {
    let waveform_legend = (0..waveform.len())
        .map(|x| x as f32 / metadata.sample_rate as f32)
        .collect();
    let waveform_trace = Scatter::new(waveform_legend, waveform)
        .mode(Mode::Lines)
        .name("time")
        .x_axis("x1")
        .y_axis("y1");
    let freq_legend = (0..freq_bins.len())
        .map(|x| x as f32 * metadata.freq_resolution)
        .collect();
    let freq_bins_trace = Scatter::new(freq_legend, freq_bins)
        .mode(Mode::Lines)
        .name("freq")
        .x_axis("x2")
        .y_axis("y2");
    let layout = Layout::new()
        .grid(
            LayoutGrid::new()
                .rows(2)
                .columns(1)
                .pattern(GridPattern::Independent)
                .row_order(RowOrder::TopToBottom),
        )
        .title(Title::new(&metadata.name))
        .x_axis(Axis::new().title(Title::new("Time (seconds)")))
        .y_axis(Axis::new().title(Title::new("Amplitude")))
        .x_axis2(Axis::new().title(Title::new("Frequency (Hz)")))
        .y_axis2(Axis::new().title(Title::new("Amplitude")))
        .width(1900)
        .height(800);
    let mut plot = Plot::new();
    plot.add_trace(waveform_trace);
    plot.add_trace(freq_bins_trace);
    plot.set_layout(layout);
    plot.write_html(file_path);
}
