use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;
    import makepad_draw::shader::draw_color::DrawColor;
    import crate::shared::widgets::*;
    import crate::shared::styles::*;

    Tooltip = {{Tooltip}}{   
        width: Fill,
        height: Fill,

        flow: Overlay
        align: {x: 0.0, y: 0.0}

        draw_bg: {
            fn pixel(self) -> vec4 {
                return vec4(0., 0., 0., 0.0)
            }
        }

        content: <View> {
            flow: Overlay
            width: Fit
            height: Fit

            <RoundedView> {
                width: Fit,
                height: Fit,
    
                padding: 16,
    
                draw_bg: {
                    color: #fff,
                    border_width: 1.0,
                    border_color: #D0D5DD,
                    radius: 2.
                }
    
                tooltip_label = <Label> {
                    width: 270,
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9},
                        text_wrap: Word,
                        color: #000
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Tooltip {
    #[rust]
    opened: bool,

    #[live]
    #[find]
    content: View,

    #[redraw]
    #[rust(DrawList2d::new(cx))]
    draw_list: DrawList2d,

    #[live]
    draw_bg: DrawQuad,
    #[layout]
    layout: Layout,
    #[walk]
    walk: Walk,
}

impl Widget for Tooltip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.content.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, _walk: Walk) -> DrawStep {
        self.draw_list.begin_overlay_reuse(cx);

        cx.begin_pass_sized_turtle(self.layout);
        self.draw_bg.begin(cx, self.walk, self.layout);

        if self.opened {
            let _ = self.content.draw_all(cx, scope);
        }

        self.draw_bg.end(cx);

        cx.end_pass_sized_turtle();
        self.draw_list.end(cx);

        DrawStep::done()
    }

    fn set_text(&mut self, text: &str) {
        self.label(id!(tooltip_label)).set_text(text);
    }
}

impl Tooltip {
    pub fn set_pos(&mut self, cx: &mut Cx, pos: DVec2) {
        self.apply_over(
            cx,
            live! {
                content: { margin: { left: (pos.x), top: (pos.y) } }
            },
        );
    }

    pub fn show(&mut self, cx: &mut Cx) {
        self.opened = true;
        self.redraw(cx);
    }

    pub fn show_with_options(&mut self, cx: &mut Cx, pos: DVec2, text: &str) {
        self.opened = true;
        self.set_text(text);
        self.set_pos(cx, pos);
        self.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.opened = false;
        self.redraw(cx);
    }
}

impl TooltipRef {
    pub fn set_text(&mut self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
        }
    }

    pub fn set_pos(&mut self, cx: &mut Cx, pos: DVec2) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_pos(cx, pos);
        }
    }

    pub fn show(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx);
        }
    }

    pub fn show_with_options(&mut self, cx: &mut Cx, pos: DVec2, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_with_options(cx, pos, text);
        }
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide(cx);
        }
    }
}
