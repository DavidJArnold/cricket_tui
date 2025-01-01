use std::{
    char,
    sync::{Arc, Mutex},
};

use cursive::{
    theme::Theme, utils::markup::StyledString, view::Resizable, views::{Button, EditView, LinearLayout, RadioGroup, TextView}, Cursive
};

use cricket_scoring::{error::BallString, scoring::{innings::Innings, player::Player, BallEvents, BallOutcome}};

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

fn parse(ball: &str) -> Result<BallOutcome, BallString> {
    // basic format is runs followed by extra events:
    //   1: 1 run
    //   .: No run
    //   W: Wicket (no runs)
    //   1X: 1 wide (equivalent to X)
    //   4X: 4 wides
    //   WX: Wicket and wide
    //   4L: 4 leg byes
    //   N: New over
    //
    // dot -> . (equivalent to 0)
    // runs -> 0, 1, 2, 3, 4, etc.
    // wicket -> W
    // wide -> X
    // no ball -> O
    // bye -> B
    // leg bye -> L
    // four -> F
    // six -> S
    //
    // empty input is not permitted
    //
    // Records must be some digits or a period, followed by up to three of W/X/B/L/O/F/S, or N
    // Valid combinations: W, WX, WB, WL, WO, X, O, OB, OL, L, B, WOF, WOS, XF, OF, OS, OBF, OLF, LF, BF.
    //
    // If no period or digits are found, it will be assumed no runs were scored.
    // Therefore, a digit must appear with B or L to indicate how many byes/leg byes.
    //
    // TODO: Prevent duplicate letters/periods, verify ordering

    let mut ball_events = vec![];

    if ball.is_empty() {
        return Err(BallString::EmptyBallString);
    }
    const ALLOWED_CHARS: [char; 8] = ['.', 'W', 'X', 'B', 'L', 'O', 'F', 'S'];
    for c in ball.chars() {
        if !(char::is_ascii_digit(&c) || ALLOWED_CHARS.contains(&c)) {
            return Err(BallString::InvalidBallStringCharacter(c));
        }
    }

    if (ball.contains('B') || ball.contains('L')) && !ball.chars().next().unwrap().is_ascii_digit()
    {
        // A bye/leg bye must include the number of runs scored
        return Err(BallString::InvalidByeCharacter);
    }

    if (ball.contains('F') && ball.contains('S')) || (ball.contains('B') && ball.contains('L')) {
        // cannot have both a four and a six, or a bye and a leg bye
        return Err(BallString::InvalidBallDescription);
    }

    if ball.contains('W') {
        ball_events.push(BallEvents::Wicket);
    } else if ball.contains('X') {
        ball_events.push(BallEvents::Wide);
    } else if ball.contains('O') {
        ball_events.push(BallEvents::NoBall);
    } else if ball.contains('L') {
        ball_events.push(BallEvents::LegBye);
    } else if ball.contains('B') {
        ball_events.push(BallEvents::Bye);
    } else if ball.contains('F') {
        ball_events.push(BallEvents::Four);
    } else if ball.contains('S') {
        ball_events.push(BallEvents::Six);
    };

    let runs = if ball.starts_with('.') {
        0
    } else {
        let runs_string = ball.matches(char::is_numeric).next();
        match runs_string {
            None => 0,
            Some(x) => x.parse::<i32>().expect("Can't convert to i32"),
        }
    };

    Ok(BallOutcome::new(runs, ball_events))
}

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(Theme::terminal_default());
    siv.add_global_callback('q', Cursive::quit);
    siv.set_user_data("");

    let runs_data: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let runs_data_edit: Arc<Mutex<String>> = Arc::clone(&runs_data);
    let runs_data_print: Arc<Mutex<String>> = Arc::clone(&runs_data);

    let mut team: Vec<Player> = vec![];
    let a: char = 'A';
    for idx in 0..11 {
        team.push(Player::new(
            String::from_utf8(vec![(a as usize + idx) as u8]).unwrap(),
        ));
    }

    let innings: Arc<Mutex<Innings>> = Arc::new(Mutex::new(Innings::new(team.clone().try_into().unwrap(), team.try_into().unwrap())));
    let innings_score = Arc::clone(&innings);
    let innings_over = Arc::clone(&innings);
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
    let (wicket, wicket_radio) = v_radio(
        "", vec![(' ', "No wicket"), ('W', "Wicket")]
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
        .child(wicket)
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
            ball_str.push(*wicket_radio.selection());
            ball_str.push(*delivery_radio.selection());
            ball_str.push(*boundary_radio.selection());
            ball_str.push(*byes_radio.selection());
            ball_str.retain(|x| x != ' ');
            match parse(&ball_str) {
                Ok(x) => match x.validate() {
                    Ok(()) => innings_score.lock().unwrap().score_ball(&x),
                    Err(_) => ball_str.push_str("Error in validation"),
                },
                Err(x) => ball_str.push_str(&format!("Error in parsing {x}")),
            };
            ball_str.push('\n');
            ball_str.push_str(&format!("{}", innings_score.lock().unwrap()));
            ball_content.set_content(&ball_str);
        }))
        .child(Button::new("Over", move |_| {
            innings_over.lock().unwrap().over();
            over_content.set_content(&format!("{}", innings_over.lock().unwrap()));
        }));

    let v_layout = LinearLayout::vertical()
        .child(h_layout)
        .child(submission)
        .child(ball_info)
        .child(over_info);
    siv.add_layer(v_layout);

    siv.run();
}
