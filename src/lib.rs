#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use egui::Id;
pub use state::{DragDropConfig, DragDropItem, DragDropResponse, DragDropUi, DragUpdate, Handle};

use crate::item::{Item, ItemResponse};
use std::hash::Hash;

mod item;
mod state;
/// Helper functions to support the drag and drop functionality
pub mod utils;

/// Helper struct for ease of use.
pub struct Dnd<'a> {
    id: Id,
    ui: &'a mut egui::Ui,
    drag_drop_ui: DragDropUi,
}

/// Main entry point for the drag and drop functionality.
/// Loads and saves it's state from egui memory.
/// Use either [Dnd::show] or [Dnd::show_vec] to display the drag and drop UI.
/// You can use [DragDropUi::with_mouse_config] or [DragDropUi::with_touch_config] to configure the drag detection.
/// Example usage:
/// ```rust;no_run
/// use std::hash::Hash;
/// use eframe::egui;
/// use egui::CentralPanel;
/// use egui_dnd::dnd;
///
/// pub fn main() -> eframe::Result<()> {
///     let mut items = vec!["alfred", "bernhard", "christian"];
///
///     eframe::run_simple_native("DnD Simple Example", Default::default(), move |ctx, _frame| {
///         CentralPanel::default().show(ctx, |ui| {
///
///             dnd(ui, "dnd_example")
///                 .show_vec(&mut items, |ui, item, handle, state| {
///                     handle.ui(ui, |ui| {
///                         ui.label("drag");
///                     });
///                     ui.label(*item);
///                 });
///
///         });
///     })
/// }
/// ```
pub fn dnd(ui: &mut egui::Ui, id_source: impl Hash) -> Dnd {
    let id = Id::new(id_source).with("dnd");
    let dnd_ui: DragDropUi =
        ui.data_mut(|data| (*data.get_temp_mut_or_default::<DragDropUi>(id)).clone());

    Dnd {
        id,
        ui,
        drag_drop_ui: dnd_ui,
    }
}

impl<'a> Dnd<'a> {
    /// Initialize the drag and drop UI. Same as [dnd].
    pub fn new(ui: &'a mut egui::Ui, id_source: impl Hash) -> Self {
        dnd(ui, id_source)
    }

    /// Sets the config used when dragging with the mouse or when no touch config is set
    pub fn with_mouse_config(mut self, config: DragDropConfig) -> Self {
        self.drag_drop_ui = self.drag_drop_ui.with_mouse_config(config);
        self
    }

    /// Sets the config used when dragging with touch
    /// If None, the mouse config is used instead
    /// Use [DragDropConfig::touch] or [DragDropConfig::touch_scroll] to get a config optimized for touch
    /// The default is [DragDropConfig::touch]
    /// For dragging in a ScrollArea, use [DragDropConfig::touch_scroll]
    pub fn with_touch_config(mut self, config: Option<DragDropConfig>) -> Self {
        self.drag_drop_ui = self.drag_drop_ui.with_touch_config(config);
        self
    }

    /// Display the drag and drop UI.
    /// `items` should be an iterator over items that should be sorted.
    ///
    /// The items won't be sorted automatically, but you can use [Dnd::show_vec] or [DragDropResponse::update_vec] to do so.
    /// If your items aren't in a vec, you have to sort them yourself.
    ///
    /// `item_ui` is called for each item. Display your item there.
    /// `item_ui` gets a [Handle] that can be used to display the drag handle.
    /// Only the handle can be used to drag the item. If you want the whole item to be draggable, put everything in the handle.
    pub fn show<T: DragDropItem>(
        self,
        items: impl Iterator<Item = T>,
        mut item_ui: impl FnMut(&mut egui::Ui, T, Handle, ItemState),
    ) -> DragDropResponse {
        self._show_with_inner::<T>(|id, ui, drag_drop_ui| {
            drag_drop_ui.ui(ui, items, |ui, item| {
                item.ui(ui, |ui, item, handle, state| {
                    item_ui(ui, item, handle, state)
                })
            })
        })
    }

    pub fn show_sized<T: DragDropItem>(
        self,
        items: impl Iterator<Item = T>,
        size: egui::Vec2,
        mut item_ui: impl FnMut(&mut egui::Ui, T, Handle, ItemState),
    ) -> DragDropResponse {
        self._show_with_inner::<T>(|id, ui, drag_drop_ui| {
            drag_drop_ui.ui(ui, items, |ui, item| {
                item.ui_sized(ui, size, |ui, item, handle, state| {
                    item_ui(ui, item, handle, state)
                })
            })
        })
    }

    /// Same as [Dnd::show], but automatically sorts the items.
    pub fn show_vec<T: Hash>(
        self,
        items: &mut [T],
        item_ui: impl FnMut(&mut egui::Ui, &mut T, Handle, ItemState),
    ) -> DragDropResponse {
        let response = self.show(items.iter_mut(), item_ui);
        response.update_vec(items);
        response
    }

    pub fn show_vec_sized<T: Hash>(
        self,
        items: &mut [T],
        size: egui::Vec2,
        item_ui: impl FnMut(&mut egui::Ui, &mut T, Handle, ItemState),
    ) -> DragDropResponse {
        let response = self.show_sized(items.iter_mut(), size, item_ui);
        response.update_vec(items);
        response
    }

    fn _show_with_inner<T: DragDropItem>(
        self,
        inner_fn: impl FnOnce(Id, &mut egui::Ui, &mut DragDropUi) -> DragDropResponse,
    ) -> DragDropResponse {
        let Dnd {
            id,
            ui,
            mut drag_drop_ui,
        } = self;

        let response = inner_fn(id, ui, &mut drag_drop_ui);

        ui.ctx().data_mut(|data| data.insert_temp(id, drag_drop_ui));

        response
    }
}

/// State of the current item.
pub struct ItemState {
    /// True if the item is currently being dragged.
    pub dragged: bool,
    /// Index of the item in the list.
    /// Note that when you sort the source list while the drag is still ongoing (default behaviour
    /// of [Dnd::show_vec]), this index will updated while the item is being dragged.
    /// If you sort once after the item is dropped, the index will be stable during the drag.
    pub index: usize,
}
