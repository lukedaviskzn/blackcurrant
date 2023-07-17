#[derive(Debug, Clone, Default)]
pub struct AboutModal;

impl AboutModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut close_modal = false;
        
        egui::Window::new("About Blackcurrant")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("The Blackcurrant Reception Managment System was created for the purpose of managing records of common UCT residence tasks.");
                
                // Note: non-breaking space inbetween phone number sections.
                ui.label("It was developed by Luke Davis (2023 UCR/Smuts Hall House Committee Member, Academics, IT & Media Portfolios, 071 302 5271).");

                ui.add_space(4.0);
                
                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        return close_modal;
    }
}
