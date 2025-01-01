use std::{
    char,
    sync::{Arc, Mutex},
};

use cursive::{
    theme::Theme, utils::markup::StyledString, view::Resizable, views::{Button, EditView, LinearLayout, RadioGroup, TextView}, Cursive
};

fn v_radio<T>(title: &str, btns: Vec<(char, T)>) -> (LinearLayout, RadioGroup<char>)
where
    T: Into<StyledString>,
{
    let mut radios = RadioGroup::new();
    let mut layout = LinearLayout::vertical();
    layout = layout.child(TextView::new(title));
    for (val, lbl) in btns {
        layout = layout.child(radios.button(val, lbl));
    }
    (layout, radios)
}

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(Theme::terminal_default());
    siv.add_global_callback('q', Cursive::quit);
    siv.set_user_data("");

    let runs_data: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let runs_data_edit: Arc<Mutex<String>> = Arc::clone(&runs_data);
    let runs_data_print: Arc<Mutex<String>> = Arc::clone(&runs_data);
    let runs = EditView::new()
        .max_content_width(1)
        .on_edit(move |_, v, _| {
            let r = Arc::clone(&runs_data_edit);
            let mut rs = r.lock().unwrap();

            rs.clear();
            rs.push_str(v);
        });
    let resize_runs = runs.resized(
        cursive::view::SizeConstraint::Fixed(2),
        cursive::view::SizeConstraint::Fixed(1),
    );
    let (delivery, delivery_radio) = v_radio(
        "",
        vec![(' ', "Legal delivery"), ('O', "No ball"), ('X', "Wide")],
    );
    let (boundary, boundary_radio) =
        v_radio("", vec![(' ', "no boundary"), ('F', "Four"), ('S', "Six")]);
    let (byes, byes_radio) = v_radio(
        "",
        vec![(' ', "Not byes"), ('B', "Byes"), ('L', "Leg byes")],
    );

    let h_layout = LinearLayout::horizontal()
        .child(resize_runs)
        .child(delivery)
        .child(boundary)
        .child(byes);

    let mut ball_info = TextView::new("");
    let mut over_info = TextView::new("");
    let ball_content = ball_info.get_shared_content();
    let over_content = over_info.get_shared_content();

    let submission = LinearLayout::horizontal()
        .child(Button::new("Save", move |_| {
            let mut ball_str = String::new();
            let r = Arc::clone(&runs_data_print);
            let rs = r.lock().unwrap();
            ball_str.push_str(&rs);
            ball_str.push(*delivery_radio.selection());
            ball_str.push(*boundary_radio.selection());
            ball_str.push(*byes_radio.selection());
            ball_content.set_content(ball_str);
        }))
        .child(Button::new("Over", move |_| {
            over_content.set_content("SETTNG ALSO");
        }));

    let v_layout = LinearLayout::vertical()
        .child(h_layout)
        .child(submission)
        .child(ball_info)
        .child(over_info);
    siv.add_layer(v_layout);

    siv.run();
}
