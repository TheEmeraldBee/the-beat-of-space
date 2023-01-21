#![allow(dead_code)]

use macroquad::color::{Color, WHITE};
use macroquad::input::{is_mouse_button_released, MouseButton};
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::prelude::{draw_text_ex, draw_texture_ex, DrawTextureParams, TextParams, Texture2D};
use macroquad::text::measure_text;

#[derive(Clone, Copy)]
pub struct UITemplate {
    pub color: Color,
    pub hover_color: Color,
    pub texture: Texture2D
}

impl UITemplate {
    pub fn new(texture: Texture2D, color: Color, hover_color: Option<Color>) -> Self {
        let mut chosen_hover_color = color;
        if let Some(new_hover_color) = hover_color {
            chosen_hover_color= new_hover_color;
        }

        Self {
            color,
            hover_color: chosen_hover_color,
            texture
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
            texture: self.texture.clone()
        }
    }
}

#[derive(Clone)]
pub struct UIElement {
    pub rect: Rect,
    pub color: Color,
    pub hover_color: Color,
    pub texture: Texture2D
}

impl UIElement {
    pub fn new(rect: Rect, color: Color, texture: Texture2D, hover_color: Option<Color>) -> Self {
        let mut chosen_hover_color = color;
        if let Some(new_hover_color) = hover_color {
            chosen_hover_color= new_hover_color;
        }

        Self {
            rect,
            color,
            hover_color: chosen_hover_color,
            texture
        }
    }

    pub fn from_template(rect: Rect, template: UITemplate) -> Self {
        Self {
            rect,
            color: template.color,
            hover_color: template.hover_color,
            texture: template.texture
        }
    }

    /// Returns if mouse is pressing it
    pub fn update(&self, mouse_pos: Vec2) -> bool {

        if self.is_hovering(mouse_pos) {
            draw_texture_ex(self.texture, self.rect.x, self.rect.y, self.hover_color, DrawTextureParams {
                dest_size: Some(vec2(self.rect.w, self.rect.h)),
                ..Default::default()
            });
        } else {
            draw_texture_ex(self.texture, self.rect.x, self.rect.y, self.color, DrawTextureParams {
                dest_size: Some(vec2(self.rect.w, self.rect.h)),
                ..Default::default()
            });
        }

        self.is_hovering(mouse_pos) && is_mouse_button_released(MouseButton::Left)
    }

    /// Returns if mouse if hovering it
    pub fn is_hovering(&self, mouse_pos: Vec2) -> bool {
        return mouse_pos.x < self.rect.x + self.rect.w && mouse_pos.x > self.rect.x &&
            mouse_pos.y < self.rect.y + self.rect.h && mouse_pos.y > self.rect.y
    }
}

pub fn element_template(rect: Rect, template: UITemplate, mouse_pos: Vec2) -> bool {
    let ui_element = UIElement {
        rect,
        color: template.color,
        hover_color: template.hover_color,
        texture: template.texture
    };

    ui_element.update(mouse_pos)
}

pub fn element(ui_element: UIElement, mouse_pos: Vec2) -> bool {
    ui_element.update(mouse_pos)
}

pub fn element_text(ui_element: UIElement, mouse_pos: Vec2, text: &str, params: TextParams) -> bool {
    let return_value = element(ui_element.clone(), mouse_pos);

    let center_of_rect = get_center_of_rect(ui_element.rect);

    draw_text_justified(text, center_of_rect, params, vec2(0.5, 0.5));

    return_value
}

pub fn element_text_template(rect: Rect, template: UITemplate, mouse_pos: Vec2, text: &str, params: TextParams) -> bool {
    let return_value = element_template(rect, template, mouse_pos);

    let center_of_rect = get_center_of_rect(rect.clone());

    draw_text_justified(text, center_of_rect, params, vec2(0.5, 0.5));

    return_value
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

pub struct NineSliceElement {
    pub tex: Texture2D,
    pub corner_size: Vec2,
    pub vertical_size: Vec2,
    pub horizontal_size: Vec2
}

impl NineSliceElement {
    pub fn draw(&self, rect: Rect) {
        // Top Left
        draw_texture_ex(self.tex, rect.x, rect.y, WHITE, DrawTextureParams {
            dest_size: Some(self.corner_size.clone()),
            source: Some(justify_rect(0.0, 0.0, self.corner_size.x, self.corner_size.y, vec2(0.0, 0.0))),
            ..Default::default()
        });

        // Top Right
        draw_texture_ex(self.tex, rect.x + rect.w - self.corner_size.x, rect.y, WHITE, DrawTextureParams {
            dest_size: Some(self.corner_size.clone()),
            source: Some(justify_rect(self.tex.width(), 0.0, self.corner_size.x, self.corner_size.y, vec2(1.0, 0.0))),
            ..Default::default()
        });

        // Bottom Left
        draw_texture_ex(self.tex, rect.x, rect.y + rect.h - self.corner_size.y, WHITE, DrawTextureParams {
            dest_size: Some(self.corner_size.clone()),
            source: Some(justify_rect(0.0, self.tex.height(), self.corner_size.x, self.corner_size.y, vec2(0.0, 1.0))),
            ..Default::default()
        });

        // Bottom Right
        draw_texture_ex(self.tex, rect.x + rect.w - self.corner_size.y, rect.y + rect.h - self.corner_size.y, WHITE, DrawTextureParams {
            dest_size: Some(self.corner_size.clone()),
            source: Some(justify_rect(self.tex.width(), self.tex.height(), self.corner_size.x, self.corner_size.y, vec2(1.0, 1.0))),
            ..Default::default()
        });

        let width_left = rect.w - self.corner_size.x * 2.0;
        let height_left = rect.h - self.corner_size.y * 2.0;

        // Top
        draw_texture_ex(self.tex, rect.x + self.corner_size.x, rect.y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(width_left, self.vertical_size.y)),
            source: Some(justify_rect(self.horizontal_size.x, 0.0, self.vertical_size.x, self.vertical_size.y, vec2(0.0, 0.0))),
            ..Default::default()
        });

        // Bottom
        draw_texture_ex(self.tex, rect.x + self.corner_size.x, rect.y + rect.h - self.vertical_size.y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(width_left, self.vertical_size.y)),
            source: Some(justify_rect(self.horizontal_size.x, self.tex.height(), self.vertical_size.x, self.vertical_size.y, vec2(0.0, 1.0))),
            ..Default::default()
        });

        // Left
        draw_texture_ex(self.tex, rect.x, rect.y + self.corner_size.y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(self.horizontal_size.x, height_left)),
            source: Some(justify_rect(0.0, self.corner_size.y, self.horizontal_size.x, self.horizontal_size.y, vec2(0.0, 0.0))),
            ..Default::default()
        });

        // Right
        draw_texture_ex(self.tex, rect.x + rect.w - self.horizontal_size.x, rect.y + self.corner_size.y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(self.horizontal_size.x, height_left)),
            source: Some(justify_rect(self.tex.width(), self.corner_size.y, self.horizontal_size.x, self.horizontal_size.y, vec2(1.0, 0.0))),
            ..Default::default()
        });

        // Fill
        draw_texture_ex(self.tex, rect.x + self.corner_size.x, rect.y + self.corner_size.y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(width_left, height_left)),
            source: Some(justify_rect(self.corner_size.x, self.corner_size.y, self.tex.width() - self.corner_size.x * 2.0, self.tex.height() - self.corner_size.y * 2.0, vec2(0.0, 0.0))),
            ..Default::default()
        })
    }
}
