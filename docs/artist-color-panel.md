# Color panel for illustration

The Color tab now defaults to an sRGB hue wheel with an inner saturation/value triangle.
The triangle has pure hue on the right, white at the upper left and black at the lower left.
Drag the ring to change hue, or the triangle to mix the selected hue with white and black.
The gesture stays with the control where it started, even if the pointer crosses another area.

- **Wheel / Field** switches between the circular selector and the rectangular selector.
- Click the foreground or background swatch to select which brush colour to edit. Double-click
  opens the existing full Color Picker dialog. The swap button uses the existing colour command.
- **HSB Sliders** shows or hides gradient hue, saturation and brightness controls.
- HEX and RGB fields remain editable. Colour values use the same sRGB convention as brush colours.
- **Rotate Triangle** makes the pure-hue corner point at the selected hue on the ring.
  Turn it off to keep the triangle pointing right. The selected colour stays the same.
- The **Recent Colors** checkbox shows or hides the history without clearing its colours.
  The selector uses the space released when history is hidden.
- **Recent Colors** remembers up to twelve distinct colours. A drag adds its final colour instead
  of every intermediate sample. Click a swatch to reuse it; **+** keeps the current colour explicitly.
- The circular selector grows with the panel while preserving its shape. Short panels scroll;
  the rectangular option uses the available body height instead of a fixed-height field.

Picker mode, slider/history visibility, triangle rotation, and recent colours are included in saved workspace layouts when
Remember Workspace Changes is enabled. Existing layouts without these fields keep defaults.

Automation: `ui.set {"colorPanel":{"picker":"wheel","sliders":true}}` selects the wheel;
`picker:"rectangle"` restores the rectangular selector. `background:true` edits the background. `show_recent:false` hides history and
`rotate_triangle:false` selects fixed orientation.
`ui.inspect.colorPanel` exposes the state. `ui.set` uses whole-object replacement semantics,
so omitted fields receive defaults.

This panel works with the existing upstream dock. It does not depend on bilateral panel docking
or a movable toolbar. It is an original implementation informed by artist colour-picker workflows;
it contains no MagicPicker code or artwork. HSB brightness is HSV value, not perceptual lightness.
