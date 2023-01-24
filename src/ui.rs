#![allow(dead_code)]

use macroquad::color::{Color};
use macroquad::input::{is_mouse_button_released, MouseButton};
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::prelude::{draw_text_ex, draw_texture_ex, DrawTextureParams, TextParams, Texture2D};
use macroquad::text::measure_text;

#[derive(Clone, Copy)]
pub struct UITemplate {
    color: Color,
    hover_color: Color,
    element: Element
}

impl UITemplate {
    pub fn new(element: Element, color: Color, hover_color: Option<Color>) -> Self {
        let mut chosen_hover_color = color;
        if let Some(new_hover_color) = hover_color {
            chosen_hover_color= new_hover_color;
        }

        Self {
            color,
            hover_color: chosen_hover_color,
            element
        }
    }

    pub fn change_color(&self, color: Color, hover_color: Option<Color>) -> Self {
        let mut chosen_hover_color = color;
        if let Some(new_hover_color) = hover_color {
            chosen_hover_color= new_hover_color;
        }

        Self {
            color,
            hover_color: chosen_hover_color,
            element: self.element
        }
    }
}

#[derive(Copy, Clone)]
pub struct UIElement {
    rect: Rect,
    color: Color,
    hover_color: Color,
    element: Element,
    mouse_pos: Vec2
}

impl UIElement {
    pub fn new(rect: Rect, element: Element, color: Color, hover_color: Option<Color>) -> Self {
        let mut chosen_hover_color = color;
        if let Some(new_hover_color) = hover_color {
            chosen_hover_color= new_hover_color;
        }

        Self {
            rect,
            color,
            hover_color: chosen_hover_color,
            element,
            mouse_pos: vec2(0.0, 0.0)
        }
    }

    pub fn from_template(rect: Rect, template: UITemplate) -> Self {
        Self {
            rect,
            color: template.color,
            hover_color: template.hover_color,
            element: template.element,
            mouse_pos: vec2(0.0, 0.0)
        }
    }

    /// Returns if mouse is pressing it
    pub fn update(&mut self, mouse_pos: Vec2) -> &mut Self {
        self.mouse_pos = mouse_pos;

        if self.is_hovering() {
            self.element.draw(self.rect, self.hover_color)
        } else {
            self.element.draw(self.rect, self.color)
        }

        self
    }

    pub fn clicked(&self) -> bool {
        self.is_hovering() && is_mouse_button_released(MouseButton::Left)
    }

    /// Returns if mouse if hovering it
    pub fn is_hovering(&self) -> bool {
        return self.mouse_pos.x < self.rect.x + self.rect.w && self.mouse_pos.x > self.rect.x &&
            self.mouse_pos.y < self.rect.y + self.rect.h && self.mouse_pos.y > self.rect.y
    }
}

pub fn element_template(rect: Rect, template: UITemplate, mouse_pos: Vec2) -> UIElement {
    let mut ui_element = UIElement {
        rect,
        color: template.color,
        hover_color: template.hover_color,
        element: template.element,
        mouse_pos: vec2(0.0, 0.0)
    };

    ui_element.update(mouse_pos).clone()
}

pub fn element(mut ui_element: UIElement, mouse_pos: Vec2) -> UIElement {
    ui_element.update(mouse_pos).clone()
}

pub fn element_text(ui_element: UIElement, mouse_pos: Vec2, text: &str, params: TextParams) -> UIElement {
    let return_value = element(ui_element.clone(), mouse_pos);

    let center_of_rect = get_center_of_rect(ui_element.rect);

    draw_text_justified(text, center_of_rect, params, vec2(0.5, 0.5));

    return_value
}

pub fn element_text_template(rect: Rect, template: UITemplate, mouse_pos: Vec2, text: &str, params: TextParams) -> UIElement {
    let return_value = element_template(rect, template, mouse_pos);

    let center_of_rect = get_center_of_rect(rect.clone());

    draw_text_justified(text, center_of_rect, params, vec2(0.5, 0.5));

    return_value
}

pub fn hover_rect(rect: Rect, mouse_pos: Vec2) -> bool {
    mouse_pos.x < rect.x + rect.w && mouse_pos.x > rect.x &&
        mouse_pos.y < rect.y + rect.h && mouse_pos.y > rect.y
}

/// Justification is a 0 - 1 value for both axis where 0 is left/top and 1 is right/bottom
pub fn draw_text_justified(text: &str, pos: Vec2, params: TextParams, justification: Vec2) {
    let text_size = measure_text(text, Some(params.font), params.font_size, params.font_scale);
    let draw_pos = vec2(pos.x - text_size.width * justification.x, pos.y + text_size.height * justification.y);

    draw_text_ex(text, draw_pos.x, draw_pos.y, params);
}

/// Justification is a 0 - 1 value for both axis where 0 is left/top and 1 is right/bottom
pub fn justify_rect(x: f32, y: f32, width: f32, height: f32, justification: Vec2) -> Rect {
    Rect::new(x - width * justification.x, y - height * justification.y, width, height)
}

pub fn get_center_of_rect(rect: Rect) -> Vec2 {
    vec2(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)
}

#[derive(Copy, Clone)]
pub enum ElementType {
    Texture,
    NineSlice(Vec2)
}

#[derive(Copy, Clone)]
pub struct Element {
    pub tex: Texture2D,
    pub element_type: ElementType
}

impl Element {
    pub fn draw(&self, rect: Rect, color: Color) {
        match self.element_type {
            ElementType::Texture => {
                draw_texture_ex(self.tex, rect.x, rect.y, color, DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                })
            }
            ElementType::NineSlice(corner_size) => {
                let vertical_size = vec2(self.tex.width() - corner_size.x * 2.0, corner_size.y);
                let horizontal_size = vec2(corner_size.x, self.tex.height() - corner_size.y * 2.0);

                // Top Left
                draw_texture_ex(self.tex, rect.x, rect.y, color, DrawTextureParams {
                    dest_size: Some(corner_size.clone()),
                    source: Some(justify_rect(0.0, 0.0, corner_size.x, corner_size.y, vec2(0.0, 0.0))),
                    ..Default::default()
                });

                // Top Right
                draw_texture_ex(self.tex, rect.x + rect.w - corner_size.x, rect.y, color, DrawTextureParams {
                    dest_size: Some(corner_size.clone()),
                    source: Some(justify_rect(self.tex.width(), 0.0, corner_size.x, corner_size.y, vec2(1.0, 0.0))),
                    ..Default::default()
                });

                // Bottom Left
                draw_texture_ex(self.tex, rect.x, rect.y + rect.h - corner_size.y, color, DrawTextureParams {
                    dest_size: Some(corner_size.clone()),
                    source: Some(justify_rect(0.0, self.tex.height(), corner_size.x, corner_size.y, vec2(0.0, 1.0))),
                    ..Default::default()
                });

                // Bottom Right
                draw_texture_ex(self.tex, rect.x + rect.w - corner_size.y, rect.y + rect.h - corner_size.y, color, DrawTextureParams {
                    dest_size: Some(corner_size.clone()),
                    source: Some(justify_rect(self.tex.width(), self.tex.height(), corner_size.x, corner_size.y, vec2(1.0, 1.0))),
                    ..Default::default()
                });

                let width_left = rect.w - corner_size.x * 2.0;
                let height_left = rect.h - corner_size.y * 2.0;

                // Top
                draw_texture_ex(self.tex, rect.x + corner_size.x, rect.y, color, DrawTextureParams {
                    dest_size: Some(vec2(width_left, vertical_size.y)),
                    source: Some(justify_rect(corner_size.x, 0.0, vertical_size.x, vertical_size.y, vec2(0.0, 0.0))),
                    ..Default::default()
                });

                // Bottom
                draw_texture_ex(self.tex, rect.x + corner_size.x, rect.y + rect.h - vertical_size.y, color, DrawTextureParams {
                    dest_size: Some(vec2(width_left, vertical_size.y)),
                    source: Some(justify_rect(corner_size.x, self.tex.height(), vertical_size.x, vertical_size.y, vec2(0.0, 1.0))),
                    ..Default::default()
                });

                // Left
                draw_texture_ex(self.tex, rect.x, rect.y + corner_size.y, color, DrawTextureParams {
                    dest_size: Some(vec2(horizontal_size.x, height_left)),
                    source: Some(justify_rect(0.0, corner_size.y, horizontal_size.x, horizontal_size.y, vec2(0.0, 0.0))),
                    ..Default::default()
                });

                // Right
                draw_texture_ex(self.tex, rect.x + rect.w - horizontal_size.x, rect.y + corner_size.y, color, DrawTextureParams {
                    dest_size: Some(vec2(horizontal_size.x, height_left)),
                    source: Some(justify_rect(self.tex.width(), corner_size.y, horizontal_size.x, horizontal_size.y, vec2(1.0, 0.0))),
                    ..Default::default()
                });

                // Fill
                draw_texture_ex(self.tex, rect.x + corner_size.x, rect.y + corner_size.y, color, DrawTextureParams {
                    dest_size: Some(vec2(width_left, height_left)),
                    source: Some(justify_rect(corner_size.x, corner_size.y, self.tex.width() - corner_size.x * 2.0, self.tex.height() - corner_size.y * 2.0, vec2(0.0, 0.0))),
                    ..Default::default()
                })
            }
        }
    }
}
