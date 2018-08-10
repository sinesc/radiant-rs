use core::{Display, Monitor, Point2, Context, Result};

/// A display builder.
///
/// Obtained from [`Display::builder()`](../struct.Display.html#method.builder).
///
/// # Examples
///
/// ```rust
/// # use radiant_rs::*;
/// let display = Display::builder().dimensions((640, 480)).vsync().title("Window!").build().unwrap();
/// ```
#[must_use]
#[derive(Clone)]
pub struct DisplayBuilder {
    pub(crate) width       : u32,
    pub(crate) height      : u32,
    pub(crate) title       : String,
    pub(crate) transparent : bool,
    pub(crate) decorations : bool,
    pub(crate) monitor     : Option<Monitor>,
    pub(crate) vsync       : bool,
    pub(crate) visible     : bool,
    pub(crate) context     : Option<Context>,
}

impl DisplayBuilder {
    /// Sets a width for the display.
    pub fn width(mut self: Self, width: u32) -> Self {
        self.width = width;
        self
    }
    /// Sets a height for the display.
    pub fn height(mut self: Self, height: u32) -> Self {
        self.height = height;
        self
    }
    /// Sets dimensions for the display.
    pub fn dimensions<T>(mut self: Self, dimensions: T) -> Self where Point2<u32>: From<T> {
        let dimensions = Point2::<u32>::from(dimensions);
        self.width = dimensions.0;
        self.height = dimensions.1;
        self
    }
    /// Sets a title for the display.
    pub fn title(mut self: Self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    /// Flags the display to be transparent.
    pub fn transparent(mut self: Self) -> Self {
        self.transparent = true;
        self
    }
    /// Flags the display to be borderless.
    pub fn borderless(mut self: Self) -> Self {
        self.decorations = false;
        self
    }
    /// Sets the monitor to create the display on.
    /// note: currently monitor cannot be aquired prior to display construction due to changes in the glium backend
    pub fn monitor(mut self: Self, monitor: Monitor) -> Self {
        self.monitor = Some(monitor);
        self
    }
    /// Flags the display to use vsync.
    pub fn vsync(mut self: Self) -> Self {
        self.vsync = true;
        self
    }
    /// Use an existing context with this display.
    pub fn context(mut self: Self, context: &Context) -> Self {
        self.context = Some(context.clone());
        self
    }
    /// Flags the display to be initialially hidden.
    pub fn hidden(mut self: Self) -> Self {
        self.visible = false;
        self
    }
    /// Returns the constructed display instance.
    pub fn build(self: Self) -> Result<Display> {
        Display::new(self)
    }
    /// Creates a new DisplayBuilder instance.
    pub(crate) fn new() -> Self {
        DisplayBuilder { ..DisplayBuilder::default() }
    }
}

impl Default for DisplayBuilder {
    fn default() -> DisplayBuilder {
        DisplayBuilder {
            width       : 640,
            height      : 480,
            title       : "".to_string(),
            transparent : false,
            decorations : true,
            monitor     : None,
            vsync       : false,
            visible     : true,
            context     : None,
        }
   }
}