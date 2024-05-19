use leptos::*;

#[component]
pub fn Loading_Triangle<W>(
    /// `children` takes the `Children` type
    show: W,
) -> impl IntoView
where
    W: Fn() -> bool + 'static,
{
    view! {
        <Show when=show>
            <svg width="200px" height="200px" viewBox="-4 -1 38 28">
                <polygon class="loading" fill="transparent" stroke="#FFFF" stroke-width="0.2" stroke-linecap="round" stroke-linejoin="round" points="15,0 30,30 0,30"></polygon>
                <polygon class="loading-thumb" fill="transparent" stroke="#FFFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" points="15,0 30,30 0,30"></polygon>
            </svg>
        </Show>
    }
}
