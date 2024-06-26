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
            /*<svg width="200px" height="200px" viewBox="-4 -1 38 28">
                <polygon class="loading" fill="transparent" stroke="#FFFF" stroke-width="0.2" stroke-linecap="round" stroke-linejoin="round" points="15,0 30,30 0,30"></polygon>
                <polygon class="loading-thumb" fill="transparent" stroke="#FFFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" points="15,0 30,30 0,30"></polygon>
            </svg>*/
            <svg 
                class="container" 
                x="0px" 
                y="0px" 
                viewBox="0 0 40 50" 
                height="200" 
                width="200" 
                preserveAspectRatio="xMidYMid meet">
                <path 
                    class="loading" 
                    fill="none" 
                    stroke-width="2" 
                    pathLength="100" 
                    stroke-linecap="round"
                    d="M29.760000000000005 18.72 c0 7.28 -3.9200000000000004 13.600000000000001 -9.840000000000002 16.96 c -2.8800000000000003 1.6800000000000002 -6.24 2.64 -9.840000000000002 2.64 c -3.6 0 -6.88 -0.96 -9.76 -2.64 c0 -7.28 3.9200000000000004 -13.52 9.840000000000002 -16.96 c2.8800000000000003 -1.6800000000000002 6.24 -2.64 9.76 -2.64 S26.880000000000003 17.040000000000003 29.760000000000005 18.72 c5.84 3.3600000000000003 9.76 9.68 9.840000000000002 16.96 c -2.8800000000000003 1.6800000000000002 -6.24 2.64 -9.76 2.64 c -3.6 0 -6.88 -0.96 -9.840000000000002 -2.64 c -5.84 -3.3600000000000003 -9.76 -9.68 -9.76 -16.96 c0 -7.28 3.9200000000000004 -13.600000000000001 9.76 -16.96 C25.84 5.120000000000001 29.760000000000005 11.440000000000001 29.760000000000005 18.72z">
                </path>
                <path 
                    class="loading-thumb" 
                    fill="none" 
                    stroke-width="2" 
                    pathLength="100" 
                    stroke-linecap="round"
                    d="M29.760000000000005 18.72 c0 7.28 -3.9200000000000004 13.600000000000001 -9.840000000000002 16.96 c -2.8800000000000003 1.6800000000000002 -6.24 2.64 -9.840000000000002 2.64 c -3.6 0 -6.88 -0.96 -9.76 -2.64 c0 -7.28 3.9200000000000004 -13.52 9.840000000000002 -16.96 c2.8800000000000003 -1.6800000000000002 6.24 -2.64 9.76 -2.64 S26.880000000000003 17.040000000000003 29.760000000000005 18.72 c5.84 3.3600000000000003 9.76 9.68 9.840000000000002 16.96 c -2.8800000000000003 1.6800000000000002 -6.24 2.64 -9.76 2.64 c -3.6 0 -6.88 -0.96 -9.840000000000002 -2.64 c -5.84 -3.3600000000000003 -9.76 -9.68 -9.76 -16.96 c0 -7.28 3.9200000000000004 -13.600000000000001 9.76 -16.96 C25.84 5.120000000000001 29.760000000000005 11.440000000000001 29.760000000000005 18.72z">
                </path>
            </svg>
        </Show>
    }
}
