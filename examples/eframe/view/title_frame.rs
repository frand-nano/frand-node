use eframe::egui::*;

pub trait TitleFrame {
    fn title_frame<R>(
        &mut self, 
        title: impl Into<WidgetText>, 
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R>;
}

impl TitleFrame for Ui {
    fn title_frame<R>(
        &mut self, 
        title: impl Into<WidgetText>, 
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        Frame::default()
        .inner_margin(8.0)
        .outer_margin(4.0)
        .rounding(8.0)
        .stroke(self.visuals().widgets.noninteractive.bg_stroke)
        .rounding(self.visuals().widgets.noninteractive.rounding)
        .show(self, |ui| {   
            let id: Id = "content_width".into();

            let content_width = ui.data_mut(|data| 
                data.get_persisted::<f32>(id)
            );     

            let title_width = match content_width {
                Some(content_width) => {
                    ui.set_width(content_width);
                    ui.vertical_centered(|ui| 
                        ui.label(title).rect.width()
                    );
                    content_width
                },
                None => ui.label(title).rect.width(),
            };

            let content_width = content_width.unwrap_or_default().max(title_width);
            ui.set_max_width(content_width);
            
            let InnerResponse { inner, response } = ui.vertical(|ui| {
                ui.vertical_centered(|ui| {
                    ui.set_max_width(content_width - 4.0);
                    ui.separator()
                });
                contents(ui)     
            });

            let content_width = response.rect.width();

            ui.data_mut(|data| 
                data.insert_persisted(id, content_width)
            );

            inner
        })
    }
}