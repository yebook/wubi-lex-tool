//! Pure geometry used to persist and restore the main window.

use crate::config::WindowBounds;

pub const MIN_LOGICAL_WIDTH: u32 = 1_024;
pub const MIN_LOGICAL_HEIGHT: u32 = 640;

const MIN_SCALE_FACTOR: f64 = 0.5;
const MAX_SCALE_FACTOR: f64 = 8.0;
const MAX_LOGICAL_POSITION: i64 = 1_000_000;
const MAX_LOGICAL_SIZE: i64 = 32_768;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorWorkArea {
    pub rect: PhysicalRect,
    pub scale_factor: f64,
    pub primary: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoredPlacement {
    pub rect: PhysicalRect,
    pub monitor_index: usize,
}

pub fn logical_from_physical(rect: PhysicalRect, scale_factor: f64) -> Option<WindowBounds> {
    if !valid_scale(scale_factor) || rect.width == 0 || rect.height == 0 {
        return None;
    }

    let x = rounded_i64(f64::from(rect.x) / scale_factor)?;
    let y = rounded_i64(f64::from(rect.y) / scale_factor)?;
    let width = rounded_i64(f64::from(rect.width) / scale_factor)?;
    let height = rounded_i64(f64::from(rect.height) / scale_factor)?;
    if !(-MAX_LOGICAL_POSITION..=MAX_LOGICAL_POSITION).contains(&x)
        || !(-MAX_LOGICAL_POSITION..=MAX_LOGICAL_POSITION).contains(&y)
        || !(1..=MAX_LOGICAL_SIZE).contains(&width)
        || !(1..=MAX_LOGICAL_SIZE).contains(&height)
    {
        return None;
    }

    Some(WindowBounds {
        x: i32::try_from(x).ok()?,
        y: i32::try_from(y).ok()?,
        width: u32::try_from(width).ok()?,
        height: u32::try_from(height).ok()?,
        scale_factor,
    })
}

pub fn restore_to_work_area(
    saved: &WindowBounds,
    work_areas: &[MonitorWorkArea],
) -> Option<RestoredPlacement> {
    if !valid_saved_bounds(saved) {
        return None;
    }

    let saved_physical = PhysicalRect {
        x: scaled_i32(saved.x, saved.scale_factor)?,
        y: scaled_i32(saved.y, saved.scale_factor)?,
        width: scaled_u32(saved.width, saved.scale_factor)?,
        height: scaled_u32(saved.height, saved.scale_factor)?,
    };
    let (monitor_index, intersects) = select_monitor(saved_physical, work_areas)?;
    let target = work_areas[monitor_index];

    let min_width = scaled_u32(MIN_LOGICAL_WIDTH, target.scale_factor)?;
    let min_height = scaled_u32(MIN_LOGICAL_HEIGHT, target.scale_factor)?;
    let desired_width = scaled_u32(saved.width.max(MIN_LOGICAL_WIDTH), target.scale_factor)?;
    let desired_height = scaled_u32(saved.height.max(MIN_LOGICAL_HEIGHT), target.scale_factor)?;
    let width = fit_dimension(desired_width, min_width, target.rect.width);
    let height = fit_dimension(desired_height, min_height, target.rect.height);

    let (x, y) = if intersects {
        (
            clamp_axis(
                i64::from(saved_physical.x),
                target.rect.x,
                target.rect.width,
                width,
            ),
            clamp_axis(
                i64::from(saved_physical.y),
                target.rect.y,
                target.rect.height,
                height,
            ),
        )
    } else {
        (
            center_axis(target.rect.x, target.rect.width, width),
            center_axis(target.rect.y, target.rect.height, height),
        )
    };

    Some(RestoredPlacement {
        rect: PhysicalRect {
            x,
            y,
            width,
            height,
        },
        monitor_index,
    })
}

fn select_monitor(saved: PhysicalRect, work_areas: &[MonitorWorkArea]) -> Option<(usize, bool)> {
    let mut best: Option<(usize, i64, bool)> = None;
    for (index, monitor) in work_areas.iter().enumerate() {
        if !valid_work_area(monitor) {
            continue;
        }
        let area = intersection_area(saved, monitor.rect);
        let replace = best.is_none_or(|(_, best_area, best_primary)| {
            area > best_area || (area == best_area && monitor.primary && !best_primary)
        });
        if replace {
            best = Some((index, area, monitor.primary));
        }
    }

    let (best_index, best_area, _) = best?;
    if best_area > 0 {
        return Some((best_index, true));
    }
    work_areas
        .iter()
        .enumerate()
        .find(|(_, monitor)| monitor.primary && valid_work_area(monitor))
        .or_else(|| {
            work_areas
                .iter()
                .enumerate()
                .find(|(_, monitor)| valid_work_area(monitor))
        })
        .map(|(index, _)| (index, false))
}

fn intersection_area(left: PhysicalRect, right: PhysicalRect) -> i64 {
    let left_edge = i64::from(left.x).max(i64::from(right.x));
    let top_edge = i64::from(left.y).max(i64::from(right.y));
    let right_edge = rect_right(left).min(rect_right(right));
    let bottom_edge = rect_bottom(left).min(rect_bottom(right));
    let width = right_edge.saturating_sub(left_edge).max(0);
    let height = bottom_edge.saturating_sub(top_edge).max(0);
    width.saturating_mul(height)
}

fn rect_right(rect: PhysicalRect) -> i64 {
    i64::from(rect.x).saturating_add(i64::from(rect.width))
}

fn rect_bottom(rect: PhysicalRect) -> i64 {
    i64::from(rect.y).saturating_add(i64::from(rect.height))
}

fn fit_dimension(desired: u32, minimum: u32, available: u32) -> u32 {
    if available >= minimum {
        desired.clamp(minimum, available)
    } else {
        minimum
    }
}

fn clamp_axis(position: i64, origin: i32, available: u32, size: u32) -> i32 {
    if size > available {
        return origin;
    }
    let minimum = i64::from(origin);
    let maximum = minimum
        .saturating_add(i64::from(available))
        .saturating_sub(i64::from(size));
    saturating_i32(position.clamp(minimum, maximum))
}

fn center_axis(origin: i32, available: u32, size: u32) -> i32 {
    if size > available {
        return origin;
    }
    let offset = i64::from(available.saturating_sub(size)) / 2;
    saturating_i32(i64::from(origin).saturating_add(offset))
}

fn valid_saved_bounds(bounds: &WindowBounds) -> bool {
    valid_scale(bounds.scale_factor)
        && (-1_000_000..=1_000_000).contains(&bounds.x)
        && (-1_000_000..=1_000_000).contains(&bounds.y)
        && (1..=32_768).contains(&bounds.width)
        && (1..=32_768).contains(&bounds.height)
}

fn valid_work_area(work_area: &MonitorWorkArea) -> bool {
    valid_scale(work_area.scale_factor) && work_area.rect.width > 0 && work_area.rect.height > 0
}

fn valid_scale(scale_factor: f64) -> bool {
    scale_factor.is_finite() && (MIN_SCALE_FACTOR..=MAX_SCALE_FACTOR).contains(&scale_factor)
}

fn scaled_i32(value: i32, scale_factor: f64) -> Option<i32> {
    let value = rounded_i64(f64::from(value) * scale_factor)?;
    i32::try_from(value).ok()
}

fn scaled_u32(value: u32, scale_factor: f64) -> Option<u32> {
    let value = rounded_i64(f64::from(value) * scale_factor)?;
    u32::try_from(value).ok().filter(|value| *value > 0)
}

fn rounded_i64(value: f64) -> Option<i64> {
    if !value.is_finite() || value < i64::MIN as f64 || value > i64::MAX as f64 {
        return None;
    }
    Some(value.round() as i64)
}

fn saturating_i32(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
}

#[cfg(test)]
mod tests {
    use super::{
        MIN_LOGICAL_HEIGHT, MIN_LOGICAL_WIDTH, MonitorWorkArea, PhysicalRect,
        logical_from_physical, restore_to_work_area,
    };
    use crate::config::WindowBounds;

    fn bounds(x: i32, y: i32, width: u32, height: u32, scale_factor: f64) -> WindowBounds {
        WindowBounds {
            x,
            y,
            width,
            height,
            scale_factor,
        }
    }

    fn monitor(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        scale_factor: f64,
        primary: bool,
    ) -> MonitorWorkArea {
        MonitorWorkArea {
            rect: PhysicalRect {
                x,
                y,
                width,
                height,
            },
            scale_factor,
            primary,
        }
    }

    #[test]
    fn physical_sampling_round_trips_logical_bounds() {
        let logical = logical_from_physical(
            PhysicalRect {
                x: -300,
                y: 150,
                width: 1_536,
                height: 960,
            },
            1.5,
        )
        .expect("valid physical bounds");
        assert_eq!(logical, bounds(-200, 100, 1_024, 640, 1.5));
    }

    #[test]
    fn physical_sampling_rejects_invalid_or_unbounded_values() {
        let rect = PhysicalRect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
        assert!(logical_from_physical(rect, f64::NAN).is_none());
        assert!(logical_from_physical(rect, 0.0).is_none());
        assert!(
            logical_from_physical(
                PhysicalRect {
                    width: u32::MAX,
                    ..rect
                },
                0.5,
            )
            .is_none()
        );
    }

    #[test]
    fn restore_keeps_a_visible_single_screen_window() {
        let restored = restore_to_work_area(
            &bounds(100, 80, 1_100, 700, 1.0),
            &[monitor(0, 0, 1_920, 1_040, 1.0, true)],
        )
        .expect("placement");
        assert_eq!(restored.monitor_index, 0);
        assert_eq!(
            restored.rect,
            PhysicalRect {
                x: 100,
                y: 80,
                width: 1_100,
                height: 700
            }
        );
    }

    #[test]
    fn restore_selects_the_largest_intersection_and_supports_negative_coordinates() {
        let restored = restore_to_work_area(
            &bounds(-1_500, 100, 1_200, 700, 1.0),
            &[
                monitor(0, 0, 1_920, 1_040, 1.0, true),
                monitor(-1_920, 0, 1_920, 1_040, 1.0, false),
            ],
        )
        .expect("placement");
        assert_eq!(restored.monitor_index, 1);
        assert_eq!(restored.rect.x, -1_500);
    }

    #[test]
    fn restore_reprojects_logical_size_to_the_current_monitor_scale() {
        let restored = restore_to_work_area(
            &bounds(100, 100, 1_024, 640, 1.0),
            &[monitor(0, 0, 3_840, 2_080, 2.0, true)],
        )
        .expect("placement");
        assert_eq!(restored.rect.width, 2_048);
        assert_eq!(restored.rect.height, 1_280);
    }

    #[test]
    fn completely_offscreen_bounds_center_on_primary() {
        let restored = restore_to_work_area(
            &bounds(50_000, 50_000, 1_024, 640, 1.0),
            &[
                monitor(-1_280, 0, 1_280, 720, 1.0, false),
                monitor(0, 0, 1_920, 1_040, 1.0, true),
            ],
        )
        .expect("placement");
        assert_eq!(restored.monitor_index, 1);
        assert_eq!(restored.rect.x, 448);
        assert_eq!(restored.rect.y, 200);
    }

    #[test]
    fn oversized_and_undersized_bounds_are_fitted_to_the_work_area() {
        let oversized = restore_to_work_area(
            &bounds(10, 10, 4_000, 3_000, 1.0),
            &[monitor(0, 0, 1_600, 900, 1.0, true)],
        )
        .expect("placement");
        assert_eq!(oversized.rect.width, 1_600);
        assert_eq!(oversized.rect.height, 900);

        let undersized = restore_to_work_area(
            &bounds(10, 10, 100, 100, 1.0),
            &[monitor(0, 0, 1_600, 900, 1.0, true)],
        )
        .expect("placement");
        assert_eq!(undersized.rect.width, MIN_LOGICAL_WIDTH);
        assert_eq!(undersized.rect.height, MIN_LOGICAL_HEIGHT);
    }

    #[test]
    fn work_area_smaller_than_native_minimum_keeps_title_bar_at_origin() {
        let restored = restore_to_work_area(
            &bounds(100, 100, 1_024, 640, 1.0),
            &[monitor(-800, 50, 800, 500, 1.0, true)],
        )
        .expect("placement");
        assert_eq!(restored.rect.x, -800);
        assert_eq!(restored.rect.y, 50);
        assert_eq!(restored.rect.width, MIN_LOGICAL_WIDTH);
        assert_eq!(restored.rect.height, MIN_LOGICAL_HEIGHT);
    }

    #[test]
    fn invalid_or_missing_monitors_return_no_placement() {
        let saved = bounds(0, 0, 1_024, 640, 1.0);
        assert!(restore_to_work_area(&saved, &[]).is_none());
        assert!(
            restore_to_work_area(&saved, &[monitor(0, 0, 1_920, 1_080, f64::INFINITY, true)])
                .is_none()
        );
    }
}
