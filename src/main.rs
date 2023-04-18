use std::{cmp, collections::HashMap};

use cursive::{
    align::HAlign,
    theme::{self, ColorStyle},
    view::{
        scroll::{self, Core},
        Resizable,
    },
    views::{LinearLayout, TextView},
    Printer, Vec2, View, XY,
};


fn main() {
    let mut siv = cursive::default();
    let cv = TableView;
    siv.add_layer(cv.fixed_height(45).fixed_width(45));
    siv.run();
}

fn main() {
    let mut siv = cursive::default();
    let cv = TableView;
    siv.add_layer(cv.fixed_height(45).fixed_width(45));
    siv.run();
}
// this is the api part
#[derive(Copy, Clone)]
enum ColumnDefinition {
    Name,
    Value,
}

struct ColumnData {
    name: String,
    value: String,
}

impl ColumnView<ColumnDefinition> for ColumnData {
    fn to_column(&self, pcd: ColumnDefinition) -> String {
        match pcd {
            ColumnDefinition::Name => self.name.to_string(),
            ColumnDefinition::Value => self.name.to_string(),
        }
    }
}
// api part ends here
pub struct TableView<T, H> {
    scroll_core: Core,
    needs_relayout: bool,

    columns: Vec<TableColumn<H>>,
    column_indicies: HashMap<H, usize>,

    items: Vec<T>,
    rows_to_items: Vec<usize>,
}

cursive::impl_scroller!(TableView < T, H > ::scroll_core);

impl<T, H> Default for TableView<T, H>
where
    T: ColumnView<H>,
    H: Copy + Clone + 'static,
{
    /// Creates a new empty `TableView` without any columns.
    ///
    /// See [`TableView::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl<T, H> TableView<T, H>
where
    T: ColumnView<H>,
    H: Copy + Clone,
{
    pub fn new() -> Self {
        Self {
            scroll_core: scroll::Core::new(),
            needs_relayout: true,

            columns: Vec::new(),
            column_indicies: HashMap::new(),

            items: Vec::new(),
            rows_to_items: Vec::new(),
        }
    }

    pub fn column<S: Into<String>, C: FnOnce(TableColumn<H>) -> TableColumn<H>>(
        mut self,
        column: H,
        title: S,
        callback: C,
    ) -> Self {
        self.add_column(column, title, callback);
        self
    }

    pub fn add_column<S: Into<String>, C: FnOnce(TableColumn<H>) -> TableColumn<H>>(
        &mut self,
        column: H,
        title: S,
        callback: C,
    ) {
        self.insert_column(self.columns.len(), column, title, callback);
    }

    fn insert_column<S: Into<String>, C: FnOnce(TableColumn<H>) -> TableColumn<H>>(
        &self,
        i: usize,
        column: H,
        title: S,
        callback: C,
    ) {
        for column in &self.columns[i..] {
            *self.column_indicies.get_mut(&column.column).unwrap() += 1;
        }
        self.column_indicies.insert(column, i);
        self.columns
            .insert(i, callback(TableColumn::new(column, title.into())));

        self.needs_relayout = true;
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.rows_to_items.clear();
        self.need_relayout = true;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    /// Sets the contained items of the table.
    ///
    /// The currently active sort order is preserved and will be applied to all
    /// items.
    pub fn set_items(&mut self, items: Vec<T>) {
        self.set_items_at(items, 0);
    }

    fn set_items_at(&mut self, items: Vec<T>, new_location: usize) {
        self.items = items;
        self.rows_to_items = Vec::with_capacity(self.items.len());

        for i in 0..self.items.len() {
            self.rows_to_items.push(i);
        }
        self.needs_relayout = true;
    }
    // Sets the contained items of the table.
    ///
    /// The order of the items will be preserved even when the table is sorted.
    ///
    /// Chainable variant.
    pub fn items(self, items: Vec<T>) -> Self {
        self.with(|t| t.set_items(items))
    }

    /// Returns a immmutable reference to the item at the specified index
    /// within the underlying storage vector.
    pub fn borrow_item(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// Returns a mutable reference to the item at the specified index within
    /// the underlying storage vector.
    pub fn borrow_item_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)
    }

    /// Returns a immmutable reference to the items contained within the table.
    pub fn borrow_items(&mut self) -> &[T] {
        &self.items
    }

    /// Returns a mutable reference to the items contained within the table.
    ///
    /// Can be used to modify the items in place.
    pub fn borrow_items_mut(&mut self) -> &mut [T] {
        self.needs_relayout = true;
        &mut self.items
    }

    /// Returns the index of the currently selected item within the underlying
    /// storage vector.
    pub fn item(&self) -> Option<usize> {
        self.rows_to_items.get(self.focus).copied()
    }
    /// Inserts a new item into the table.
    ///
    /// The currently active sort order is preserved and will be applied to the
    /// newly inserted item.
    ///
    /// If no sort option is set, the item will be added to the end of the table.
    pub fn insert_item(&mut self, item: T) {
        self.insert_item_at(self.items.len(), item);
    }

    /// Inserts a new item into the table.
    ///
    /// The currently active sort order is preserved and will be applied to the
    /// newly inserted item.
    ///
    /// If no sort option is set, the item will be inserted at the given index.
    ///
    /// # Panics
    ///
    /// If `index > self.len()`.
    pub fn insert_item_at(&mut self, index: usize, item: T) {
        self.items.push(item);
        // Here we know self.items.len() > 0
        self.rows_to_items.insert(index, self.items.len() - 1);
        self.needs_relayout = true;
    }
}

impl<T, H> TableView<T, H>
where
    T: ColumnView<H>,
    H: Copy + Clone,
{
    fn draw_columns<C: Fn(&Printer, &TableColumn<H>)>(
        &self,
        printer: &Printer,
        sep: &str,
        callback: C,
    ) {
        let mut column_offset = 0;
        let column_count = self.columns.count();
        for (index, column) in self.columns.iter.enumerate() {
            let printer = &printer.offset((column_offset, 0)).focused(true);

            callback(printer, column);

            if 1 + index < column_count {
                printer.print((column.width + 1, 0), sep);
            }
            column_offset += column.width + 3;
        }
    }

    fn draw_item(&self, printer: &Printer, i: usize) {
        self.draw_columns(printer, "┆ ", |printer, column| {
            let value = self.items[self.rows_to_items[i]].to_column(column.column);
            column.draw_row(printer, value.as_str());
        });
    }

    fn draw_content(&self, printer: &Printer) {
        for i in 0..self.rows_to_items.len() {
            let printer = printer.offset((0, i));
            let color = ColorStyle::primary();
            if i < self.items.len() {
                printer.with_color(color, |printer| {
                    self.draw_item(printer, i);
                });
            }
        }
    }

    fn layout_content(&mut self, size: Vec2) {
        let column_count = self.columns.len();

        // Split up all columns into sized / unsized groups
        let (mut sized, mut usized): (Vec<&mut TableColumn<H>>, Vec<&mut TableColumn<H>>) = self
            .columns
            .iter_mut()
            .partition(|c| c.requested_width.is_some());

        // Subtract one for the seperators between our columns (that's column_count - 1)
        let available_width = size.x.saturating_sub(column_count.saturating_sub(1) * 3);

        // Calculate widths for all requested columns
        let mut remaining_width = available_width;
        for column in &mut sized {
            column.width = match *column.requested_width.as_ref().unwrap() {
                TableColumnWidth::Percent(width) => cmp::min(
                    (size.x as f32 / 100.0 * width as f32).ceil() as usize,
                    remaining_width,
                ),
                TableColumnWidth::Absolute(width) => width,
            };
            remaining_width = remaining_width.saturating_sub(column.width);
        }

        // Spread the remaining with across the unsized columns
        let remaining_columns = usized.len();
        for column in &mut usized {
            column.width = (remaining_width as f32 / remaining_columns as f32).floor() as usize;
        }

        self.needs_relayout = false;
    }

    fn content_required_size(&mut self, req: Vec2) -> Vec2 {
        Vec2::new(req.x, self.rows_to_items.len())
    }
}

trait ColumnView<H>
where
    H: Copy + Clone,
{
    fn to_column(&self, parameter: H) -> String;
}

enum TableColumnWidth {
    Percent(usize),
    Absolute(usize),
}

struct TableColumn<H> {
    column: H,
    title: String,
    alignment: HAlign,
    width: usize,
    requested_width: Option<TableColumnWidth>,
}

impl<H: Copy + Clone + 'static> TableColumn<H> {
    /// Sets the horizontal text alignment of the column.
    pub fn align(mut self, alignment: HAlign) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets how many characters of width this column will try to occupy.
    pub fn width(mut self, width: usize) -> Self {
        self.requested_width = Some(TableColumnWidth::Absolute(width));
        self
    }
    /// Sets what percentage of the width of the entire table this column will
    /// try to occupy.
    pub fn width_percent(mut self, width: usize) -> Self {
        self.requested_width = Some(TableColumnWidth::Percent(width));
        self
    }

    fn new(column: H, title: String) -> Self {
        Self {
            column,
            title,
            alignment: HAlign::Left,
            width: 0,
            requested_width: None,
        }
    }

    fn draw_header(&self, printer: &Printer) {
        let header = match self.alignment {
            HAlign::Left => format!(
                "{:<width$} ",
                self.title,
                width = self.width.saturating_sub(4)
            ),
            HAlign::Right => format!(
                "{:>width$} ",
                self.title,
                width = self.width.saturating_sub(4)
            ),
            HAlign::Center => format!(
                "{:^width$} ",
                self.title,
                width = self.width.saturating_sub(4)
            ),
        };
        printer.print((0, 0), header.as_str());
    }

    fn draw_row(&self, printer: &Printer, value: &str) {
        let value = match self.alignment {
            HAlign::Left => format!("{:<width$} ", value, width = self.width),
            HAlign::Right => format!("{:>width$} ", value, width = self.width),
            HAlign::Center => format!("{:^width$} ", value, width = self.width),
        };

        printer.print((0, 0), value.as_str());
    }
}

impl TableView {}

impl View for TableView {
    fn draw(&self, printer: &Printer) {
        let color = ColorStyle::highlight();
        printer.with_color(color, |printer| {
            self.draw_header(printer);
        });
        // printer.print(XY::new(15, 0), "something very important!");

        // printer.print(XY::new(0, 1), "something very important!");
        // printer.print(XY::new(0, 2), "something very important!");
    }
}
