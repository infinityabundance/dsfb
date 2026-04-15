use dsfb_gray::{build_public_evaluation, render_public_evaluation_report};

fn main() {
    let bundle = build_public_evaluation();
    print!("{}", render_public_evaluation_report(&bundle));
}
