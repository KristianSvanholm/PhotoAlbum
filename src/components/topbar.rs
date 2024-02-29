use leptos::*;

#[component]
pub fn TopBar() -> impl IntoView {
    // All routes accessible from navigation bar
    view! {
        <nav>
            <a href="/">"Family Album"</a> // TODO Set to Admin defined name
            <a href="upload">"Upload"</a>
        </nav>
    }
}
