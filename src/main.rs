use leptos::{mount::mount_to_body, prelude::*};

#[component]
fn App() -> impl IntoView {
    view! {
        <main>
            <h1>Leptos Starter</h1>
            <p>Ready to build.</p>
        </main>
    }
}

fn main() {
    mount_to_body(App);
}
