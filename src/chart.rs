use crate::{
    aspect_ratio::{AspectRatioCalc, CalcUsing},
    debug::DebugRect,
    inner::InnerLayout,
    layout::{HorizontalLayout, Layout, VerticalLayout},
    overlay::tooltip::Tooltip,
    projection::Projection,
    series::{RenderData, UseData},
    state::{PreState, State},
    use_watched_node::{use_watched_node, UseWatchedNode},
    AspectRatio, Font, Padding,
};
use leptos::{html::Div, *};
use std::rc::Rc;

#[derive(Clone)]
pub struct Chart<X: 'static, Y: 'static> {
    top: Vec<Rc<dyn HorizontalLayout<X, Y>>>,
    right: Vec<Rc<dyn VerticalLayout<X, Y>>>,
    bottom: Vec<Rc<dyn HorizontalLayout<X, Y>>>,
    left: Vec<Rc<dyn VerticalLayout<X, Y>>>,
    series: UseData<X, Y>,
}

impl<X, Y> Chart<X, Y> {
    pub fn new(series: UseData<X, Y>) -> Self {
        Self {
            top: vec![],
            right: vec![],
            bottom: vec![],
            left: vec![],
            series,
        }
    }

    pub fn top(mut self, opt: impl HorizontalLayout<X, Y> + 'static) -> Self {
        self.top.push(Rc::new(opt));
        self
    }

    pub fn right(mut self, opt: impl VerticalLayout<X, Y> + 'static) -> Self {
        self.right.push(Rc::new(opt));
        self
    }

    pub fn bottom(mut self, opt: impl HorizontalLayout<X, Y> + 'static) -> Self {
        self.bottom.push(Rc::new(opt));
        self
    }

    pub fn left(mut self, opt: impl VerticalLayout<X, Y> + 'static) -> Self {
        self.left.push(Rc::new(opt));
        self
    }
}

#[component]
pub fn Chart<X: Clone + PartialEq + 'static, Y: Clone + PartialEq + 'static>(
    chart: Chart<X, Y>,
    #[prop(into)] aspect_ratio: MaybeSignal<AspectRatio>,
    #[prop(into)] font: MaybeSignal<Font>,
    #[prop(into, optional)] debug: MaybeSignal<bool>,
    #[prop(into, optional)] padding: Option<MaybeSignal<Padding>>,
    #[prop(optional)] inner: Vec<InnerLayout<X, Y>>,
    #[prop(into, optional)] tooltip: Option<Tooltip<X, Y>>,
) -> impl IntoView {
    let root = create_node_ref::<Div>();
    let watch = use_watched_node(root);

    // Aspect ratio signal
    let have_dimensions = create_memo(move |_| watch.bounds.get().is_some());
    let width = create_memo(move |_| watch.bounds.get().unwrap_or_default().width());
    let height = create_memo(move |_| watch.bounds.get().unwrap_or_default().height());
    let calc = create_memo(move |_| match aspect_ratio.get().0 {
        CalcUsing::Env(calc) => calc.mk_signal(width, height),
        CalcUsing::Known(calc) => calc,
    });

    let debug = create_memo(move |_| debug.get());
    let padding = create_memo(move |_| {
        padding
            .map(|p| p.get())
            .unwrap_or_else(move || Padding::from(font.get().width()))
    });

    view! {
        <div class="_chartistry" node_ref=root style="width: fit-content; height: fit-content; overflow: visible;">
            <DebugRect label="Chart" debug=debug />
            <Show when=move || have_dimensions.get() fallback=|| view!(<p>"Loading..."</p>)>
                <RenderChart
                    chart=chart.clone()
                    watch=watch.clone()
                    debug=debug
                    aspect_ratio=calc
                    font=move || font.get()
                    padding=move || padding.get()
                    inner=inner.clone()
                    tooltip=tooltip.clone()
                />
            </Show>
        </div>
    }
}

#[component]
fn RenderChart<X: Clone + PartialEq + 'static, Y: Clone + PartialEq + 'static>(
    chart: Chart<X, Y>,
    watch: UseWatchedNode,
    #[prop(into)] debug: Signal<bool>,
    aspect_ratio: Memo<AspectRatioCalc>,
    #[prop(into)] font: Signal<Font>,
    #[prop(into)] padding: Signal<Padding>,
    inner: Vec<InnerLayout<X, Y>>,
    tooltip: Option<Tooltip<X, Y>>,
) -> impl IntoView {
    let Chart {
        mut top,
        right,
        bottom,
        mut left,
        series: data,
    } = chart;

    // Edges are added top to bottom, left to right. Layout compoeses inside out:
    top.reverse();
    left.reverse();

    // Compose edges
    let pre = PreState::new(debug, font, padding, data.clone());
    let (layout, edges) = Layout::compose(top, right, bottom, left, aspect_ratio, &pre);

    // Finalise state
    let projection = {
        let inner = layout.inner;
        let position_range = data.position_range;
        create_memo(move |_| Projection::new(inner.get(), position_range.get())).into()
    };
    let state = State::new(pre, &watch, layout, projection);

    // Render edges
    let edges = edges
        .into_iter()
        .map(|r| r.render(state.clone()))
        .collect_view();

    // Inner
    let inner = inner
        .into_iter()
        .map(|opt| opt.into_use(&state).render(state.clone()))
        .collect_view();

    let outer = state.layout.outer;
    view! {
        <svg
            width=move || format!("{}px", outer.get().width())
            height=move || format!("{}px", outer.get().height())
            viewBox=move || with!(|outer| format!("0 0 {} {}", outer.width(), outer.height()))
            style="display: block; overflow: visible;">
            <DebugRect label="RenderChart" debug=debug bounds=vec![outer.into()] />
            {inner}
            {edges}
            <RenderData data=data state=state.clone() />
        </svg>
        {tooltip.map(|tooltip| view! {
            <Tooltip tooltip=tooltip state=state />
        })}
    }
}
