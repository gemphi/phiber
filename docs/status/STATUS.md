# Phiber status against Blender

Generated from the code by `manual::status`. Do not edit it by hand: after a change, run
`UPDATE_STATUS_DOC=1 cargo test -p phiber --lib manual::status` to write it again.

| Word | Means |
|---|---|
| Wired | The menu entry runs something. Whether that operation is complete is not checked here. |
| Not available yet | The entry only reports "not available yet". |
| Applies | A rule takes the edited value into the scene or the app. |
| Read by | No rule takes it, but the file named reads the field. |
| Shown only | Nothing reads the field: it shows a value and an edit changes nothing. |
| Rendered / Ignored / n/a | The Render tab's own audit: the renderer uses it, does not, or has nothing to use it on. |

## Summary

| Component | Shown in | Entries or fields | Working | Not yet |
|---|---|---:|---:|---:|
| Top Bar menus | Layout, UV Lights, Texturing, Shading, Animation, Rendering, ECS Wand, Geometry Nodes, Scripting | 49 | 49 | 0 |
| 3D Viewport menus | Layout, UV Lights, Texturing, Shading, Animation, Geometry Nodes, Scripting | 77 | 71 | 6 |
| Outliner menus | Layout, UV Lights, Texturing, Shading, Animation, ECS Wand, Geometry Nodes, Scripting | 3 | 3 | 0 |
| Inspector menus | Layout, UV Lights, Texturing, Shading, Animation, Rendering, ECS Wand, Geometry Nodes, Scripting | 3 | 3 | 0 |
| Timeline menus | Layout, Animation, Rendering, ECS Wand, Geometry Nodes | 20 | 14 | 6 |
| File Browser menus | Layout, Shading | 10 | 1 | 9 |
| Image Editor menus | UV Lights, Texturing, Shading, Rendering | 49 | 34 | 15 |
| Dope Sheet menus | Layout, Animation, Rendering, ECS Wand, Geometry Nodes | 140 | 65 | 75 |
| NLA menus | Layout, Animation, Rendering, ECS Wand, Geometry Nodes | 76 | 25 | 51 |
| Node Editor menus | Shading | 13 | 8 | 5 |
| UV Editor menus | UV Lights, Texturing, Shading | 70 | 39 | 31 |
| Compositor menus | Shading, ECS Wand | 13 | 8 | 5 |
| Shader Editor menus | Shading | 9 | 8 | 1 |
| Geometry Nodes menus | Shading, Geometry Nodes | 13 | 8 | 5 |
| Drivers menus | Layout, Animation, Rendering, ECS Wand, Geometry Nodes | 146 | 44 | 102 |
| Asset Browser menus | Layout, Shading | 15 | 4 | 11 |
| Graph Editor menus | Layout, Animation, Rendering, ECS Wand, Geometry Nodes | 166 | 57 | 109 |
| Preferences menus |  | 6 | 3 | 3 |
| Properties ▸ Active Tool | | 22 | 16 | 6 |
| Properties ▸ Render | | 41 | 28 | 13 |
| Properties ▸ Output | | 41 | 16 | 25 |
| Properties ▸ View Layer | | 34 | 14 | 20 |
| Properties ▸ Scene | | 21 | 0 | 21 |
| Properties ▸ World | | 11 | 2 | 9 |
| Properties ▸ Collection | | 6 | 0 | 6 |
| Properties ▸ Object | | 32 | 7 | 25 |
| Properties ▸ Modifiers | | 2 | 0 | 2 |
| Properties ▸ Particles | | 21 | 3 | 18 |
| Properties ▸ Physics | | 16 | 0 | 16 |
| Properties ▸ Constraints | | 2 | 0 | 2 |
| Properties ▸ Data | | 12 | 0 | 12 |
| Properties ▸ Material | | 20 | 7 | 13 |
| Properties ▸ Texture | | 23 | 3 | 20 |

## Editors

### Top Bar

Shown in: Layout, UV Lights, Texturing, Shading, Animation, Rendering, ECS Wand, Geometry Nodes, Scripting.

| Menu entry | Shortcut | Status |
|---|---|---|
| Phiber ▸ Welcome... |  | Wired |
| Phiber ▸ About Phiber |  | Wired |
| Phiber ▸ Load Factory Settings |  | Wired |
| Phiber ▸ Quit | Ctrl Q | Wired |
| File ▸ New ▸ General | Ctrl N | Wired |
| File ▸ New ▸ Shading |  | Wired |
| File ▸ New ▸ UV Lights |  | Wired |
| File ▸ New ▸ Animation |  | Wired |
| File ▸ New ▸ Rendering |  | Wired |
| File ▸ New ▸ ECS Wand |  | Wired |
| File ▸ New ▸ Geometry Nodes |  | Wired |
| File ▸ New ▸ Scripting |  | Wired |
| File ▸ Open... | Ctrl O | Wired |
| File ▸ Open Recent |  | Wired |
| File ▸ Save | Ctrl S | Wired |
| File ▸ Save As... | Shift Ctrl S | Wired |
| File ▸ Versions ▸ Save Incremental | Ctrl Alt S | Wired |
| File ▸ Versions ▸ Save Copy... |  | Wired |
| File ▸ Versions ▸ Revert |  | Wired |
| File ▸ Import ▸ Universal Scene Description (.usd*) |  | Wired |
| File ▸ Import ▸ Wavefront (.obj) |  | Wired |
| File ▸ Import ▸ glTF 2.0 (.glb/.gltf) |  | Wired |
| File ▸ Export ▸ Universal Scene Description (.usda) |  | Wired |
| File ▸ Export ▸ Wavefront (.obj) |  | Wired |
| File ▸ Export ▸ Stanford PLY (.ply) |  | Wired |
| File ▸ Export ▸ glTF 2.0 (.gltf) |  | Wired |
| File ▸ Export ▸ STL (.stl) |  | Wired |
| File ▸ Export ▸ Export All Collections |  | Wired |
| Edit ▸ Undo | Ctrl Z | Wired |
| Edit ▸ Redo | Shift Ctrl Z | Wired |
| Edit ▸ Undo History |  | Wired |
| Edit ▸ Search... | F3 | Wired |
| Edit ▸ Rename Active Item... | F2 | Wired |
| Edit ▸ Preferences... | Ctrl , | Wired |
| Render ▸ Render Image | F12 | Wired |
| Render ▸ Render Animation | Ctrl F12 | Wired |
| Render ▸ Render Turnaround |  | Wired |
| Render ▸ Render Audio... |  | Wired |
| Render ▸ View Render | F11 | Wired |
| Render ▸ View Animation | Ctrl F11 | Wired |
| Render ▸ Lock Interface |  | Wired |
| Window ▸ Toggle Window Fullscreen |  | Wired |
| Window ▸ Next Workspace | Ctrl Page Down | Wired |
| Window ▸ Previous Workspace | Ctrl Page Up | Wired |
| Window ▸ Show Status Bar |  | Wired |
| Window ▸ Space Tabs |  | Wired |
| Help ▸ Manual |  | Wired |
| Help ▸ Report a Bug |  | Wired |
| Help ▸ About Phiber |  | Wired |

### 3D Viewport

Shown in: Layout, UV Lights, Texturing, Shading, Animation, Geometry Nodes, Scripting.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Toolbar | T | Wired |
| View ▸ Sidebar | N | Wired |
| View ▸ Frame Selected | Numpad . | Wired |
| View ▸ Frame All | Home | Wired |
| View ▸ Perspective/Orthographic | Numpad 5 | Wired |
| View ▸ Viewpoint ▸ Camera | Numpad 0 | Wired |
| View ▸ Viewpoint ▸ Top | Numpad 7 | Wired |
| View ▸ Viewpoint ▸ Bottom | Ctrl Numpad 7 | Wired |
| View ▸ Viewpoint ▸ Front | Numpad 1 | Wired |
| View ▸ Viewpoint ▸ Back | Ctrl Numpad 1 | Wired |
| View ▸ Viewpoint ▸ Right | Numpad 3 | Wired |
| View ▸ Viewpoint ▸ Left | Ctrl Numpad 3 | Wired |
| View ▸ Navigation ▸ Orbit Left | Numpad 4 | Wired |
| View ▸ Navigation ▸ Orbit Right | Numpad 6 | Wired |
| View ▸ Navigation ▸ Orbit Up | Numpad 8 | Wired |
| View ▸ Navigation ▸ Orbit Down | Numpad 2 | Wired |
| View ▸ Navigation ▸ Zoom In | Numpad + | Wired |
| View ▸ Navigation ▸ Zoom Out | Numpad - | Wired |
| View ▸ Align View ▸ Center Cursor and Frame All | Shift C | Wired |
| View ▸ View Regions ▸ Clipping Region... | Alt B | Not available yet |
| View ▸ View Regions ▸ Render Region... | Ctrl B | Wired |
| View ▸ View Regions ▸ Clear Render Region | Ctrl Alt B | Wired |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Wired |
| Select ▸ None | Alt A | Wired |
| Select ▸ Invert | Ctrl I | Wired |
| Select ▸ Box Select | B | Wired |
| Select ▸ Circle Select | C | Wired |
| Add ▸ Mesh ▸ Plane |  | Wired |
| Add ▸ Mesh ▸ Cube |  | Wired |
| Add ▸ Mesh ▸ Circle |  | Wired |
| Add ▸ Mesh ▸ UV Sphere |  | Wired |
| Add ▸ Mesh ▸ Ico Sphere |  | Wired |
| Add ▸ Mesh ▸ Cylinder |  | Wired |
| Add ▸ Mesh ▸ Cone |  | Wired |
| Add ▸ Mesh ▸ Torus |  | Wired |
| Add ▸ Mesh ▸ Monkey |  | Wired |
| Add ▸ Curve |  | Not available yet |
| Add ▸ Empty ▸ Plain Axes |  | Wired |
| Add ▸ Empty ▸ Arrows |  | Wired |
| Add ▸ Empty ▸ Single Arrow |  | Wired |
| Add ▸ Empty ▸ Circle |  | Wired |
| Add ▸ Empty ▸ Cube |  | Wired |
| Add ▸ Empty ▸ Sphere |  | Wired |
| Add ▸ Empty ▸ Cone |  | Wired |
| Add ▸ Light ▸ Point |  | Wired |
| Add ▸ Light ▸ Sun |  | Wired |
| Add ▸ Light ▸ Spot |  | Wired |
| Add ▸ Light ▸ Area |  | Wired |
| Add ▸ Camera |  | Wired |
| Object ▸ Transform ▸ Move | G | Wired |
| Object ▸ Transform ▸ Rotate | R | Wired |
| Object ▸ Transform ▸ Scale | S | Wired |
| Object ▸ Clear ▸ Location | Alt G | Wired |
| Object ▸ Clear ▸ Rotation | Alt R | Wired |
| Object ▸ Clear ▸ Scale | Alt S | Wired |
| Object ▸ Snap ▸ Selection to Grid |  | Wired |
| Object ▸ Snap ▸ Selection to Cursor |  | Wired |
| Object ▸ Snap ▸ Selection to Active |  | Wired |
| Object ▸ Snap ▸ Cursor to Selected |  | Wired |
| Object ▸ Snap ▸ Cursor to World Origin |  | Wired |
| Object ▸ Snap ▸ Cursor to Grid |  | Wired |
| Object ▸ Snap ▸ Cursor to Active |  | Wired |
| Object ▸ Shade Auto Smooth |  | Not available yet |
| Object ▸ Animation ▸ Insert Keyframe | I | Wired |
| Object ▸ Animation ▸ Insert Keyframe with Keying Set | K | Wired |
| Object ▸ Animation ▸ Delete Keyframes... | Alt I | Wired |
| Object ▸ Animation ▸ Clear Keyframes... |  | Wired |
| Object ▸ Animation ▸ Change Keying Set... | Shift K | Not available yet |
| Object ▸ Animation ▸ Bake Action... |  | Not available yet |
| Object ▸ Animation ▸ Bake Object Transform to Grease Pencil... |  | Not available yet |
| Object ▸ Parent | Ctrl P | Wired |
| Object ▸ Clear Parent | Alt P | Wired |
| Object ▸ Duplicate Objects | Shift D | Wired |
| Object ▸ Delete | X | Wired |
| Object ▸ Show/Hide ▸ Show Hidden Objects | Alt H | Wired |
| Object ▸ Show/Hide ▸ Hide Selected | H | Wired |

### Outliner

Shown in: Layout, UV Lights, Texturing, Shading, Animation, ECS Wand, Geometry Nodes, Scripting.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Show Hierarchy | Home | Wired |
| View ▸ Expand One Level | Numpad + | Wired |
| View ▸ Collapse One Level | Numpad - | Wired |

### Inspector

Shown in: Layout, UV Lights, Texturing, Shading, Animation, Rendering, ECS Wand, Geometry Nodes, Scripting.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Expand All |  | Wired |
| View ▸ Collapse All |  | Wired |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |

### Timeline

Shown in: Layout, Animation, Rendering, ECS Wand, Geometry Nodes.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Sidebar | N | Wired |
| View ▸ Channels |  | Not available yet |
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Scene Range |  | Wired |
| View ▸ Go to Start Frame | Shift Left | Wired |
| View ▸ Show Markers |  | Wired |
| View ▸ Show Seconds | Ctrl T | Not available yet |
| View ▸ Sync Visible Range |  | Not available yet |
| View ▸ Only Keyframes from Selected Channels |  | Not available yet |
| View ▸ Only Show Errors |  | Not available yet |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Marker ▸ Lock Markers |  | Wired |
| Marker ▸ Jump to Previous Marker |  | Wired |
| Marker ▸ Jump to Next Marker |  | Wired |
| Marker ▸ Bind Camera to Markers | Ctrl B | Not available yet |
| Marker ▸ Move Marker | G | Wired |
| Marker ▸ Rename Marker | F2 | Wired |
| Marker ▸ Delete Marker |  | Wired |
| Marker ▸ Duplicate Marker |  | Wired |
| Marker ▸ Add Marker | M | Wired |

### File Browser

Shown in: Layout, Shading.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Source List |  | Not available yet |
| View ▸ File Path |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Display Size |  | Not available yet |
| View ▸ Recursion Level |  | Not available yet |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Not available yet |
| Select ▸ None | Alt A | Not available yet |
| Select ▸ Invert | Ctrl I | Not available yet |
| Select ▸ Box Select | B | Not available yet |

### Image Editor

Shown in: UV Lights, Texturing, Shading, Rendering.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Toolbar | T | Wired |
| View ▸ Sidebar | N | Wired |
| View ▸ Tool Settings |  | Not available yet |
| View ▸ Asset Shelf |  | Not available yet |
| View ▸ Adjust Last Operation |  | Not available yet |
| View ▸ Update Automatically |  | Not available yet |
| View ▸ Show Metadata |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Frame All | Home | Wired |
| View ▸ Center View to Cursor |  | Not available yet |
| View ▸ Zoom ▸ 12.5% (1:8) |  | Wired |
| View ▸ Zoom ▸ 25% (1:4) |  | Wired |
| View ▸ Zoom ▸ 50% (1:2) |  | Wired |
| View ▸ Zoom ▸ 100% (1:1) | Numpad 1 | Wired |
| View ▸ Zoom ▸ 200% (2:1) | Numpad 2 | Wired |
| View ▸ Zoom ▸ 400% (4:1) | Numpad 4 | Wired |
| View ▸ Zoom ▸ 800% (8:1) | Numpad 8 | Wired |
| View ▸ Zoom ▸ Zoom In | Numpad + | Wired |
| View ▸ Zoom ▸ Zoom Out | Numpad - | Wired |
| View ▸ Set Render Region | Ctrl B | Wired |
| View ▸ Clear Render Region | Ctrl Alt B | Wired |
| View ▸ Render Slot Cycle Next | J | Wired |
| View ▸ Render Slot Cycle Previous | Alt J | Wired |
| View ▸ Show Same Material |  | Not available yet |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Image ▸ New... | Alt N | Wired |
| Image ▸ Open... | Alt O | Wired |
| Image ▸ Open Cached Render |  | Not available yet |
| Image ▸ Replace... |  | Not available yet |
| Image ▸ Reload | Alt R | Wired |
| Image ▸ Edit Externally |  | Not available yet |
| Image ▸ Copy | Ctrl C | Wired |
| Image ▸ Paste | Ctrl V | Not available yet |
| Image ▸ Save | Alt S | Wired |
| Image ▸ Save As... | Shift Alt S | Wired |
| Image ▸ Save a Copy... |  | Wired |
| Image ▸ Save Sequence |  | Not available yet |
| Image ▸ Save All Images |  | Not available yet |
| Image ▸ Invert ▸ Invert Image Colors |  | Wired |
| Image ▸ Invert ▸ Invert Red Channel |  | Wired |
| Image ▸ Invert ▸ Invert Green Channel |  | Wired |
| Image ▸ Invert ▸ Invert Blue Channel |  | Wired |
| Image ▸ Invert ▸ Invert Alpha Channel |  | Wired |
| Image ▸ Resize |  | Not available yet |
| Image ▸ Transform ▸ Flip Horizontally |  | Wired |
| Image ▸ Transform ▸ Flip Vertically |  | Wired |
| Image ▸ Transform ▸ Rotate 90° Clockwise |  | Wired |
| Image ▸ Transform ▸ Rotate 90° Counter-Clockwise |  | Wired |
| Image ▸ Transform ▸ Rotate 180° |  | Wired |

### Dope Sheet

Shown in: Layout, Animation, Rendering, ECS Wand, Geometry Nodes.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Sidebar | N | Wired |
| View ▸ Adjust Last Operation |  | Not available yet |
| View ▸ Channels |  | Not available yet |
| View ▸ Playback Controls |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Scene Range |  | Wired |
| View ▸ Go to Current Frame | Numpad 0 | Wired |
| View ▸ Multi-Word Match Search |  | Not available yet |
| View ▸ Realtime Updates |  | Not available yet |
| View ▸ Show Sliders |  | Not available yet |
| View ▸ Show Handles and Interpolation |  | Not available yet |
| View ▸ Show Curve Extremes |  | Not available yet |
| View ▸ Auto-Merge Keyframes |  | Not available yet |
| View ▸ Show Markers |  | Wired |
| View ▸ Use Timecode | Ctrl T | Not available yet |
| View ▸ Sync Visible Range |  | Not available yet |
| View ▸ Set Preview Range | P | Not available yet |
| View ▸ Clear Preview Range | Alt P | Wired |
| View ▸ Set Preview Range to Selected | Ctrl Alt P | Wired |
| View ▸ Toggle Graph Editor | Ctrl Tab | Wired |
| View ▸ Cache ▸ Show Cache |  | Not available yet |
| View ▸ Cache ▸ Softbody |  | Not available yet |
| View ▸ Cache ▸ Particles |  | Not available yet |
| View ▸ Cache ▸ Cloth |  | Not available yet |
| View ▸ Cache ▸ Simulation Nodes |  | Not available yet |
| View ▸ Cache ▸ Smoke |  | Not available yet |
| View ▸ Cache ▸ Dynamic Paint |  | Not available yet |
| View ▸ Cache ▸ Rigid Body |  | Not available yet |
| View ▸ Area ▸ Horizontal Split |  | Not available yet |
| View ▸ Area ▸ Vertical Split |  | Not available yet |
| View ▸ Area ▸ Maximize Area | Ctrl Space | Wired |
| View ▸ Area ▸ Focus Mode | Ctrl Alt Space | Not available yet |
| View ▸ Area ▸ Duplicate Area into New Window |  | Not available yet |
| View ▸ Area ▸ Close Area |  | Not available yet |
| Select ▸ All | A | Wired |
| Select ▸ None | Alt A | Wired |
| Select ▸ Invert | Ctrl I | Wired |
| Select ▸ Box Select | B | Not available yet |
| Select ▸ Box Select (Axis Range) | Alt B | Not available yet |
| Select ▸ Circle Select | C | Not available yet |
| Select ▸ Lasso Select ▸ Set |  | Not available yet |
| Select ▸ Lasso Select ▸ Extend |  | Not available yet |
| Select ▸ Lasso Select ▸ Subtract |  | Not available yet |
| Select ▸ More | Ctrl Numpad + | Not available yet |
| Select ▸ Less | Ctrl Numpad - | Not available yet |
| Select ▸ Select Linked | L | Not available yet |
| Select ▸ Columns on Selected Keys | K | Not available yet |
| Select ▸ Column on Current Frame | Ctrl K | Not available yet |
| Select ▸ Columns on Selected Markers | Shift K | Not available yet |
| Select ▸ Between Selected Markers | Alt K | Not available yet |
| Select ▸ Before Current Frame | [ | Not available yet |
| Select ▸ After Current Frame | ] | Not available yet |
| Marker ▸ Lock Markers |  | Wired |
| Marker ▸ Jump to Previous Marker |  | Wired |
| Marker ▸ Jump to Next Marker |  | Wired |
| Marker ▸ Bind Camera to Markers | Ctrl B | Not available yet |
| Marker ▸ Move Marker | G | Wired |
| Marker ▸ Rename Marker | F2 | Wired |
| Marker ▸ Delete Marker |  | Wired |
| Marker ▸ Duplicate Marker |  | Wired |
| Marker ▸ Add Marker | M | Wired |
| Channel ▸ Delete Channels | X | Not available yet |
| Channel ▸ Clean Channels |  | Wired |
| Channel ▸ Group Channels | Ctrl G | Not available yet |
| Channel ▸ Ungroup Channels | Ctrl Alt G | Not available yet |
| Channel ▸ Toggle Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Toggle Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Enable Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Enable Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Disable Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Disable Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Toggle Channel Editability | Tab | Not available yet |
| Channel ▸ Extrapolation Mode ▸ Constant Extrapolation |  | Wired |
| Channel ▸ Extrapolation Mode ▸ Linear Extrapolation |  | Wired |
| Channel ▸ Extrapolation Mode ▸ Make Cyclic (F-Modifier) |  | Not available yet |
| Channel ▸ Extrapolation Mode ▸ Clear Cyclic (F-Modifier) |  | Not available yet |
| Channel ▸ Expand Channels | Numpad + | Not available yet |
| Channel ▸ Collapse Channels | Numpad - | Not available yet |
| Channel ▸ Move... ▸ To Top | Shift PageUp | Not available yet |
| Channel ▸ Move... ▸ Up | PageUp | Not available yet |
| Channel ▸ Move... ▸ Down | PageDown | Not available yet |
| Channel ▸ Move... ▸ To Bottom | Shift PageDown | Not available yet |
| Channel ▸ Revive Disabled F-Curves |  | Not available yet |
| Channel ▸ Bake Channels |  | Not available yet |
| Channel ▸ Frame Selected Channels |  | Not available yet |
| Key ▸ Transform ▸ Move | G | Wired |
| Key ▸ Transform ▸ Extend | E | Not available yet |
| Key ▸ Transform ▸ Slide | Shift T | Not available yet |
| Key ▸ Transform ▸ Scale | S | Not available yet |
| Key ▸ Snap ▸ Selection to Current Frame |  | Wired |
| Key ▸ Snap ▸ Selection to Nearest Frame |  | Wired |
| Key ▸ Snap ▸ Selection to Nearest Second |  | Wired |
| Key ▸ Snap ▸ Selection to Nearest Marker |  | Wired |
| Key ▸ Mirror ▸ By Times Over Current Frame |  | Wired |
| Key ▸ Mirror ▸ By Values Over Zero Value |  | Not available yet |
| Key ▸ Mirror ▸ By Times Over First Selected Marker |  | Wired |
| Key ▸ Insert Keyframes | I | Wired |
| Key ▸ Jump to Keyframes | Ctrl G | Wired |
| Key ▸ Copy Keyframes | Ctrl C | Not available yet |
| Key ▸ Paste Keyframes | Ctrl V | Not available yet |
| Key ▸ Paste Flipped | Shift Ctrl V | Not available yet |
| Key ▸ Duplicate | Shift D | Wired |
| Key ▸ Delete Keyframes | X | Wired |
| Key ▸ Keyframe Type ▸ Keyframe |  | Not available yet |
| Key ▸ Keyframe Type ▸ Breakdown |  | Not available yet |
| Key ▸ Keyframe Type ▸ Moving Hold |  | Not available yet |
| Key ▸ Keyframe Type ▸ Extreme |  | Not available yet |
| Key ▸ Keyframe Type ▸ Jitter |  | Not available yet |
| Key ▸ Keyframe Type ▸ Generated |  | Not available yet |
| Key ▸ Handle Type ▸ Free |  | Wired |
| Key ▸ Handle Type ▸ Aligned |  | Wired |
| Key ▸ Handle Type ▸ Vector |  | Wired |
| Key ▸ Handle Type ▸ Automatic |  | Wired |
| Key ▸ Handle Type ▸ Auto Clamped |  | Wired |
| Key ▸ Interpolation Mode ▸ Constant |  | Wired |
| Key ▸ Interpolation Mode ▸ Linear |  | Wired |
| Key ▸ Interpolation Mode ▸ Bézier |  | Wired |
| Key ▸ Interpolation Mode ▸ Sinusoidal |  | Wired |
| Key ▸ Interpolation Mode ▸ Quadratic |  | Wired |
| Key ▸ Interpolation Mode ▸ Cubic |  | Wired |
| Key ▸ Interpolation Mode ▸ Quartic |  | Wired |
| Key ▸ Interpolation Mode ▸ Quintic |  | Wired |
| Key ▸ Interpolation Mode ▸ Exponential |  | Wired |
| Key ▸ Interpolation Mode ▸ Circular |  | Wired |
| Key ▸ Interpolation Mode ▸ Back |  | Wired |
| Key ▸ Interpolation Mode ▸ Bounce |  | Wired |
| Key ▸ Interpolation Mode ▸ Elastic |  | Wired |
| Key ▸ Easing Mode ▸ Automatic Easing |  | Wired |
| Key ▸ Easing Mode ▸ Ease In |  | Wired |
| Key ▸ Easing Mode ▸ Ease Out |  | Wired |
| Key ▸ Easing Mode ▸ Ease In and Out |  | Wired |
| Key ▸ Clean Keyframes |  | Wired |
| Key ▸ Bake Keyframes | Shift Alt O | Not available yet |
| Key ▸ Discontinuity (Euler) Filter |  | Not available yet |
| Action ▸ Merge Animation |  | Not available yet |
| Action ▸ Separate Slots |  | Not available yet |
| Action ▸ Move Slots to new Action |  | Not available yet |
| Action ▸ Push Down Action |  | Wired |
| Action ▸ Stash Action |  | Wired |

### NLA

Shown in: Layout, Animation, Rendering, ECS Wand, Geometry Nodes.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Sidebar | N | Wired |
| View ▸ Adjust Last Operation |  | Not available yet |
| View ▸ Channels |  | Not available yet |
| View ▸ Playback Controls |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Scene Range |  | Wired |
| View ▸ Go to Current Frame | Numpad 0 | Wired |
| View ▸ Realtime Updates |  | Not available yet |
| View ▸ Show Control F-Curves |  | Not available yet |
| View ▸ Show Markers |  | Wired |
| View ▸ Show Local Markers |  | Not available yet |
| View ▸ Use Timecode | Ctrl T | Not available yet |
| View ▸ Sync Visible Range |  | Not available yet |
| View ▸ Set Preview Range | P | Not available yet |
| View ▸ Clear Preview Range | Alt P | Wired |
| View ▸ Set Preview Range to Selected | Ctrl Alt P | Wired |
| View ▸ Area ▸ Horizontal Split |  | Not available yet |
| View ▸ Area ▸ Vertical Split |  | Not available yet |
| View ▸ Area ▸ Maximize Area | Ctrl Space | Wired |
| View ▸ Area ▸ Focus Mode | Ctrl Alt Space | Not available yet |
| View ▸ Area ▸ Duplicate Area into New Window |  | Not available yet |
| View ▸ Area ▸ Close Area |  | Not available yet |
| Select ▸ All | A | Wired |
| Select ▸ None | Alt A | Wired |
| Select ▸ Invert | Ctrl I | Wired |
| Select ▸ Box Select | B | Not available yet |
| Select ▸ Box Select (Axis Range) | Alt B | Not available yet |
| Select ▸ Before Current Frame | [ | Not available yet |
| Select ▸ After Current Frame | ] | Not available yet |
| Marker ▸ Lock Markers |  | Wired |
| Marker ▸ Jump to Previous Marker |  | Wired |
| Marker ▸ Jump to Next Marker |  | Wired |
| Marker ▸ Bind Camera to Markers | Ctrl B | Not available yet |
| Marker ▸ Move Marker | G | Wired |
| Marker ▸ Rename Marker | F2 | Wired |
| Marker ▸ Delete Marker |  | Wired |
| Marker ▸ Duplicate Marker |  | Wired |
| Marker ▸ Add Marker | M | Wired |
| Add ▸ Action |  | Wired |
| Add ▸ Transition | Shift T | Not available yet |
| Add ▸ Sound | Shift K | Not available yet |
| Add ▸ Selected Objects |  | Not available yet |
| Tracks ▸ Add |  | Wired |
| Tracks ▸ Add Above Selected |  | Wired |
| Tracks ▸ Delete |  | Wired |
| Tracks ▸ Move ▸ To Top | Shift PageUp | Not available yet |
| Tracks ▸ Move ▸ Up | PageUp | Not available yet |
| Tracks ▸ Move ▸ Down | PageDown | Not available yet |
| Tracks ▸ Move ▸ To Bottom | Shift PageDown | Not available yet |
| Tracks ▸ Remove Empty Animation Data |  | Not available yet |
| Strips ▸ Transform ▸ Move | G | Not available yet |
| Strips ▸ Transform ▸ Extend | E | Not available yet |
| Strips ▸ Transform ▸ Scale | S | Not available yet |
| Strips ▸ Transform ▸ Swap | Alt F | Not available yet |
| Strips ▸ Transform ▸ Move Up | PageUp | Not available yet |
| Strips ▸ Transform ▸ Move Down | PageDown | Not available yet |
| Strips ▸ Snap ▸ Selection to Current Frame |  | Not available yet |
| Strips ▸ Snap ▸ Selection to Nearest Frame |  | Not available yet |
| Strips ▸ Snap ▸ Selection to Nearest Second |  | Not available yet |
| Strips ▸ Snap ▸ Selection to Nearest Marker |  | Not available yet |
| Strips ▸ Split | Y | Not available yet |
| Strips ▸ Duplicate | Alt D | Not available yet |
| Strips ▸ Linked Duplicate | Shift D | Not available yet |
| Strips ▸ Delete | X | Wired |
| Strips ▸ Make Meta | Ctrl G | Not available yet |
| Strips ▸ Remove Meta | Ctrl Alt G | Not available yet |
| Strips ▸ Toggle Muting | H | Wired |
| Strips ▸ Bake Action |  | Not available yet |
| Strips ▸ Apply Scale | Ctrl A | Not available yet |
| Strips ▸ Clear Scale | Alt S | Not available yet |
| Strips ▸ Sync Action Length |  | Not available yet |
| Strips ▸ Make Single User | U | Not available yet |
| Strips ▸ Start Editing Stashed Action | Shift Tab | Not available yet |
| Strips ▸ Start Tweaking Strip Actions (Full Stack) | Tab | Not available yet |
| Strips ▸ Start Tweaking Strip Actions (Lower Stack) |  | Not available yet |

### Node Editor

Shown in: Shading.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Selected | Numpad . | Wired |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Wired |
| Select ▸ None | Alt A | Wired |
| Select ▸ Invert | Ctrl I | Wired |
| Add ▸ Search... |  | Not available yet |
| Add ▸ Shader |  | Not available yet |
| Add ▸ Texture |  | Not available yet |
| Add ▸ Converter |  | Not available yet |
| Node ▸ Duplicate | Shift D | Wired |
| Node ▸ Delete | X | Wired |
| Node ▸ Join in New Frame | Ctrl J | Not available yet |

### UV Editor

Shown in: UV Lights, Texturing, Shading.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Toolbar | T | Wired |
| View ▸ Sidebar | N | Wired |
| View ▸ Tool Settings |  | Not available yet |
| View ▸ Asset Shelf |  | Not available yet |
| View ▸ Adjust Last Operation |  | Not available yet |
| View ▸ Update Automatically |  | Not available yet |
| View ▸ Show Metadata |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Frame All | Home | Wired |
| View ▸ Center View to Cursor |  | Not available yet |
| View ▸ Zoom ▸ 12.5% (1:8) |  | Wired |
| View ▸ Zoom ▸ 25% (1:4) |  | Wired |
| View ▸ Zoom ▸ 50% (1:2) |  | Wired |
| View ▸ Zoom ▸ 100% (1:1) | Numpad 1 | Wired |
| View ▸ Zoom ▸ 200% (2:1) | Numpad 2 | Wired |
| View ▸ Zoom ▸ 400% (4:1) | Numpad 4 | Wired |
| View ▸ Zoom ▸ 800% (8:1) | Numpad 8 | Wired |
| View ▸ Zoom ▸ Zoom In | Numpad + | Wired |
| View ▸ Zoom ▸ Zoom Out | Numpad - | Wired |
| View ▸ Set Render Region | Ctrl B | Wired |
| View ▸ Clear Render Region | Ctrl Alt B | Wired |
| View ▸ Render Slot Cycle Next | J | Wired |
| View ▸ Render Slot Cycle Previous | Alt J | Wired |
| View ▸ Show Same Material |  | Not available yet |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Not available yet |
| Select ▸ None | Alt A | Not available yet |
| Select ▸ Invert | Ctrl I | Not available yet |
| Select ▸ Box Select | B | Not available yet |
| Select ▸ Circle Select | C | Not available yet |
| Select ▸ Select Linked | Ctrl L | Not available yet |
| Select ▸ Select Pinned | Shift P | Not available yet |
| Image ▸ New... | Alt N | Wired |
| Image ▸ Open... | Alt O | Wired |
| Image ▸ Open Cached Render |  | Not available yet |
| Image ▸ Replace... |  | Not available yet |
| Image ▸ Reload | Alt R | Wired |
| Image ▸ Edit Externally |  | Not available yet |
| Image ▸ Copy | Ctrl C | Wired |
| Image ▸ Paste | Ctrl V | Not available yet |
| Image ▸ Save | Alt S | Wired |
| Image ▸ Save As... | Shift Alt S | Wired |
| Image ▸ Save a Copy... |  | Wired |
| Image ▸ Save Sequence |  | Not available yet |
| Image ▸ Save All Images |  | Not available yet |
| Image ▸ Invert ▸ Invert Image Colors |  | Wired |
| Image ▸ Invert ▸ Invert Red Channel |  | Wired |
| Image ▸ Invert ▸ Invert Green Channel |  | Wired |
| Image ▸ Invert ▸ Invert Blue Channel |  | Wired |
| Image ▸ Invert ▸ Invert Alpha Channel |  | Wired |
| Image ▸ Resize |  | Not available yet |
| Image ▸ Transform ▸ Flip Horizontally |  | Wired |
| Image ▸ Transform ▸ Flip Vertically |  | Wired |
| Image ▸ Transform ▸ Rotate 90° Clockwise |  | Wired |
| Image ▸ Transform ▸ Rotate 90° Counter-Clockwise |  | Wired |
| Image ▸ Transform ▸ Rotate 180° |  | Wired |
| UV ▸ Transform ▸ Flip Horizontally |  | Wired |
| UV ▸ Transform ▸ Flip Vertically |  | Wired |
| UV ▸ Transform ▸ Rotate 90° Clockwise |  | Wired |
| UV ▸ Transform ▸ Rotate 90° Counter-Clockwise |  | Wired |
| UV ▸ Transform ▸ Rotate 180° |  | Wired |
| UV ▸ Mirror ▸ X Axis |  | Not available yet |
| UV ▸ Mirror ▸ Y Axis |  | Not available yet |
| UV ▸ Unwrap | U | Not available yet |
| UV ▸ Pin | P | Not available yet |
| UV ▸ Unpin | Alt P | Not available yet |
| UV ▸ Pack Islands |  | Not available yet |
| UV ▸ Average Islands Scale |  | Not available yet |
| UV ▸ Minimize Stretch |  | Not available yet |
| UV ▸ Stitch | Alt V | Not available yet |

### Compositor

Shown in: Shading, ECS Wand.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Selected | Numpad . | Wired |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Wired |
| Select ▸ None | Alt A | Wired |
| Select ▸ Invert | Ctrl I | Wired |
| Add ▸ Search... |  | Not available yet |
| Add ▸ Shader |  | Not available yet |
| Add ▸ Texture |  | Not available yet |
| Add ▸ Converter |  | Not available yet |
| Node ▸ Duplicate | Shift D | Wired |
| Node ▸ Delete | X | Wired |
| Node ▸ Join in New Frame | Ctrl J | Not available yet |

### Shader Editor

Shown in: Shading.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Selected | Numpad . | Wired |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Wired |
| Select ▸ None | Alt A | Wired |
| Select ▸ Invert | Ctrl I | Wired |
| Node ▸ Duplicate | Shift D | Wired |
| Node ▸ Delete | X | Wired |
| Node ▸ Join in New Frame | Ctrl J | Not available yet |

### Geometry Nodes

Shown in: Shading, Geometry Nodes.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Selected | Numpad . | Wired |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Wired |
| Select ▸ None | Alt A | Wired |
| Select ▸ Invert | Ctrl I | Wired |
| Add ▸ Search... |  | Not available yet |
| Add ▸ Shader |  | Not available yet |
| Add ▸ Texture |  | Not available yet |
| Add ▸ Converter |  | Not available yet |
| Node ▸ Duplicate | Shift D | Wired |
| Node ▸ Delete | X | Wired |
| Node ▸ Join in New Frame | Ctrl J | Not available yet |

### Drivers

Shown in: Layout, Animation, Rendering, ECS Wand, Geometry Nodes.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Sidebar | N | Wired |
| View ▸ Adjust Last Operation |  | Not available yet |
| View ▸ Channels |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Frame All | Home | Not available yet |
| View ▸ Go to Current Frame | Numpad 0 | Wired |
| View ▸ Realtime Updates |  | Not available yet |
| View ▸ Show Sliders |  | Not available yet |
| View ▸ Auto-Merge Keyframes |  | Not available yet |
| View ▸ Auto-Lock Key Axis |  | Not available yet |
| View ▸ Show Cursor |  | Not available yet |
| View ▸ Use Timecode | Ctrl T | Not available yet |
| View ▸ Sync Visible Range |  | Not available yet |
| View ▸ Show Extrapolation |  | Not available yet |
| View ▸ Show Handles | Ctrl H | Not available yet |
| View ▸ Only Selected Keyframes Handles |  | Not available yet |
| View ▸ Toggle Dope Sheet | Ctrl Tab | Wired |
| View ▸ Area ▸ Horizontal Split |  | Not available yet |
| View ▸ Area ▸ Vertical Split |  | Not available yet |
| View ▸ Area ▸ Maximize Area | Ctrl Space | Wired |
| View ▸ Area ▸ Focus Mode | Ctrl Alt Space | Not available yet |
| View ▸ Area ▸ Duplicate Area into New Window |  | Not available yet |
| View ▸ Area ▸ Close Area |  | Not available yet |
| Select ▸ All | A | Not available yet |
| Select ▸ None | Alt A | Not available yet |
| Select ▸ Invert | Ctrl I | Not available yet |
| Select ▸ Box Select (Include Handles) |  | Not available yet |
| Select ▸ Box Select (Axis Range) | Alt B | Not available yet |
| Select ▸ Box Select | B | Not available yet |
| Select ▸ Circle Select | C | Not available yet |
| Select ▸ Lasso Select ▸ Set |  | Not available yet |
| Select ▸ Lasso Select ▸ Extend |  | Not available yet |
| Select ▸ Lasso Select ▸ Subtract |  | Not available yet |
| Select ▸ More | Ctrl Numpad + | Not available yet |
| Select ▸ Less | Ctrl Numpad - | Not available yet |
| Select ▸ Select Linked | L | Not available yet |
| Select ▸ Columns on Selected Keys | K | Not available yet |
| Select ▸ Column on Current Frame | Ctrl K | Not available yet |
| Select ▸ Columns on Selected Markers | Shift K | Not available yet |
| Select ▸ Between Selected Markers | Alt K | Not available yet |
| Select ▸ Before Current Frame | [ | Not available yet |
| Select ▸ After Current Frame | ] | Not available yet |
| Select ▸ Select Handles |  | Not available yet |
| Select ▸ Select Key |  | Not available yet |
| Channel ▸ Delete Channels | X | Not available yet |
| Channel ▸ Delete Invalid Drivers |  | Not available yet |
| Channel ▸ Group Channels | Ctrl G | Not available yet |
| Channel ▸ Ungroup Channels | Ctrl Alt G | Not available yet |
| Channel ▸ Toggle Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Toggle Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Enable Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Enable Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Disable Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Disable Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Toggle Channel Editability | Tab | Not available yet |
| Channel ▸ Extrapolation Mode ▸ Constant Extrapolation |  | Wired |
| Channel ▸ Extrapolation Mode ▸ Linear Extrapolation |  | Wired |
| Channel ▸ Extrapolation Mode ▸ Make Cyclic (F-Modifier) |  | Not available yet |
| Channel ▸ Extrapolation Mode ▸ Clear Cyclic (F-Modifier) |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Generator |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Built-In Function |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Envelope |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Cycles |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Noise |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Limits |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Stepped Interpolation |  | Not available yet |
| Channel ▸ Hide Selected Curves | H | Not available yet |
| Channel ▸ Hide Unselected Curves | Shift H | Not available yet |
| Channel ▸ Reveal Curves | Alt H | Not available yet |
| Channel ▸ Expand Channels | Numpad + | Not available yet |
| Channel ▸ Collapse Channels | Numpad - | Not available yet |
| Channel ▸ Move... ▸ To Top | Shift PageUp | Not available yet |
| Channel ▸ Move... ▸ Up | PageUp | Not available yet |
| Channel ▸ Move... ▸ Down | PageDown | Not available yet |
| Channel ▸ Move... ▸ To Bottom | Shift PageDown | Not available yet |
| Channel ▸ Frame Selected Channels |  | Not available yet |
| Key ▸ Transform ▸ Move | G | Not available yet |
| Key ▸ Transform ▸ Extend | E | Not available yet |
| Key ▸ Transform ▸ Rotate | R | Not available yet |
| Key ▸ Transform ▸ Scale | S | Not available yet |
| Key ▸ Snap ▸ Selection to Current Frame |  | Wired |
| Key ▸ Snap ▸ Selection to Cursor Value |  | Not available yet |
| Key ▸ Snap ▸ Selection to Nearest Frame |  | Wired |
| Key ▸ Snap ▸ Selection to Nearest Second |  | Wired |
| Key ▸ Snap ▸ Selection to Nearest Marker |  | Wired |
| Key ▸ Snap ▸ Flatten Handles |  | Not available yet |
| Key ▸ Snap ▸ Equalize Handles |  | Not available yet |
| Key ▸ Snap ▸ Cursor to Selection |  | Not available yet |
| Key ▸ Snap ▸ Cursor Value to Selection |  | Not available yet |
| Key ▸ Mirror ▸ By Times Over Current Frame |  | Wired |
| Key ▸ Mirror ▸ By Values Over Cursor Value |  | Not available yet |
| Key ▸ Mirror ▸ By Times Over Zero Time |  | Wired |
| Key ▸ Mirror ▸ By Values Over Zero Value |  | Not available yet |
| Key ▸ Mirror ▸ By Times Over First Selected Marker |  | Wired |
| Key ▸ Jump to Selected | Ctrl G | Not available yet |
| Key ▸ Insert ▸ All Channels |  | Wired |
| Key ▸ Insert ▸ Only Selected Channels |  | Not available yet |
| Key ▸ Insert ▸ Only Active F-Curve |  | Not available yet |
| Key ▸ Insert ▸ Active Channels at Cursor |  | Not available yet |
| Key ▸ Insert ▸ Selected Channels at Cursor |  | Not available yet |
| Key ▸ Copy | Ctrl C | Not available yet |
| Key ▸ Paste | Ctrl V | Not available yet |
| Key ▸ Paste Flipped | Shift Ctrl V | Not available yet |
| Key ▸ Duplicate | Shift D | Wired |
| Key ▸ Delete | X | Not available yet |
| Key ▸ Handle Type ▸ Free |  | Wired |
| Key ▸ Handle Type ▸ Aligned |  | Wired |
| Key ▸ Handle Type ▸ Vector |  | Wired |
| Key ▸ Handle Type ▸ Automatic |  | Wired |
| Key ▸ Handle Type ▸ Auto Clamped |  | Wired |
| Key ▸ Interpolation Mode ▸ Constant |  | Wired |
| Key ▸ Interpolation Mode ▸ Linear |  | Wired |
| Key ▸ Interpolation Mode ▸ Bézier |  | Wired |
| Key ▸ Interpolation Mode ▸ Sinusoidal |  | Wired |
| Key ▸ Interpolation Mode ▸ Quadratic |  | Wired |
| Key ▸ Interpolation Mode ▸ Cubic |  | Wired |
| Key ▸ Interpolation Mode ▸ Quartic |  | Wired |
| Key ▸ Interpolation Mode ▸ Quintic |  | Wired |
| Key ▸ Interpolation Mode ▸ Exponential |  | Wired |
| Key ▸ Interpolation Mode ▸ Circular |  | Wired |
| Key ▸ Interpolation Mode ▸ Back |  | Wired |
| Key ▸ Interpolation Mode ▸ Bounce |  | Wired |
| Key ▸ Interpolation Mode ▸ Elastic |  | Wired |
| Key ▸ Easing Type ▸ Automatic Easing |  | Wired |
| Key ▸ Easing Type ▸ Ease In |  | Wired |
| Key ▸ Easing Type ▸ Ease Out |  | Wired |
| Key ▸ Easing Type ▸ Ease In and Out |  | Wired |
| Key ▸ Density ▸ Decimate (Ratio) |  | Not available yet |
| Key ▸ Density ▸ Decimate (Allowed Change) |  | Not available yet |
| Key ▸ Density ▸ Bake Keyframes | Shift Alt O | Not available yet |
| Key ▸ Density ▸ Clean Keyframes |  | Wired |
| Key ▸ Blend ▸ Breakdown |  | Not available yet |
| Key ▸ Blend ▸ Blend to Neighbor |  | Not available yet |
| Key ▸ Blend ▸ Blend to Default Value |  | Not available yet |
| Key ▸ Blend ▸ Ease |  | Not available yet |
| Key ▸ Blend ▸ Blend Offset |  | Not available yet |
| Key ▸ Blend ▸ Blend to Ease |  | Not available yet |
| Key ▸ Blend ▸ Match Slope |  | Not available yet |
| Key ▸ Blend ▸ Push Pull |  | Not available yet |
| Key ▸ Blend ▸ Shear Keys |  | Not available yet |
| Key ▸ Blend ▸ Scale Average |  | Not available yet |
| Key ▸ Blend ▸ Scale from Neighbor |  | Not available yet |
| Key ▸ Blend ▸ Time Offset |  | Not available yet |
| Key ▸ Smooth ▸ Smooth (Gaussian) |  | Not available yet |
| Key ▸ Smooth ▸ Smooth (Legacy) | Alt O | Not available yet |
| Key ▸ Smooth ▸ Butterworth Smooth |  | Not available yet |

### Asset Browser

Shown in: Layout, Shading.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Source List |  | Not available yet |
| View ▸ Asset Details |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Display Size |  | Not available yet |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Select ▸ All | A | Not available yet |
| Select ▸ None | Alt A | Not available yet |
| Select ▸ Invert | Ctrl I | Not available yet |
| Select ▸ Box Select | B | Not available yet |
| Library ▸ Refresh |  | Wired |
| Library ▸ Refresh Remote Listing |  | Not available yet |
| Catalog ▸ Undo |  | Wired |
| Catalog ▸ Redo |  | Wired |
| Catalog ▸ Save Catalogs |  | Not available yet |
| Catalog ▸ New Catalog |  | Not available yet |

### Graph Editor

Shown in: Layout, Animation, Rendering, ECS Wand, Geometry Nodes.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Sidebar | N | Wired |
| View ▸ Adjust Last Operation |  | Not available yet |
| View ▸ Channels |  | Not available yet |
| View ▸ Playback Controls |  | Not available yet |
| View ▸ Frame Selected | Numpad . | Not available yet |
| View ▸ Frame All | Home | Wired |
| View ▸ Frame Scene Range |  | Wired |
| View ▸ Go to Current Frame | Numpad 0 | Wired |
| View ▸ Realtime Updates |  | Not available yet |
| View ▸ Show Sliders |  | Not available yet |
| View ▸ Auto-Merge Keyframes |  | Not available yet |
| View ▸ Auto-Lock Key Axis |  | Not available yet |
| View ▸ Show Markers |  | Wired |
| View ▸ Show Cursor |  | Not available yet |
| View ▸ Use Timecode | Ctrl T | Not available yet |
| View ▸ Sync Visible Range |  | Not available yet |
| View ▸ Show Extrapolation |  | Not available yet |
| View ▸ Show Handles | Ctrl H | Not available yet |
| View ▸ Only Selected Keyframes Handles |  | Not available yet |
| View ▸ Set Preview Range | P | Not available yet |
| View ▸ Clear Preview Range | Alt P | Wired |
| View ▸ Set Preview Range to Selected | Ctrl Alt P | Wired |
| View ▸ Toggle Dope Sheet | Ctrl Tab | Wired |
| View ▸ Area ▸ Horizontal Split |  | Not available yet |
| View ▸ Area ▸ Vertical Split |  | Not available yet |
| View ▸ Area ▸ Maximize Area | Ctrl Space | Wired |
| View ▸ Area ▸ Focus Mode | Ctrl Alt Space | Not available yet |
| View ▸ Area ▸ Duplicate Area into New Window |  | Not available yet |
| View ▸ Area ▸ Close Area |  | Not available yet |
| Select ▸ All | A | Not available yet |
| Select ▸ None | Alt A | Not available yet |
| Select ▸ Invert | Ctrl I | Not available yet |
| Select ▸ Box Select (Include Handles) |  | Not available yet |
| Select ▸ Box Select (Axis Range) | Alt B | Not available yet |
| Select ▸ Box Select | B | Not available yet |
| Select ▸ Circle Select | C | Not available yet |
| Select ▸ Lasso Select ▸ Set |  | Not available yet |
| Select ▸ Lasso Select ▸ Extend |  | Not available yet |
| Select ▸ Lasso Select ▸ Subtract |  | Not available yet |
| Select ▸ More | Ctrl Numpad + | Not available yet |
| Select ▸ Less | Ctrl Numpad - | Not available yet |
| Select ▸ Select Linked | L | Not available yet |
| Select ▸ Columns on Selected Keys | K | Not available yet |
| Select ▸ Column on Current Frame | Ctrl K | Not available yet |
| Select ▸ Columns on Selected Markers | Shift K | Not available yet |
| Select ▸ Between Selected Markers | Alt K | Not available yet |
| Select ▸ Before Current Frame | [ | Not available yet |
| Select ▸ After Current Frame | ] | Not available yet |
| Select ▸ Select Handles |  | Not available yet |
| Select ▸ Select Key |  | Not available yet |
| Marker ▸ Lock Markers |  | Wired |
| Marker ▸ Jump to Previous Marker |  | Wired |
| Marker ▸ Jump to Next Marker |  | Wired |
| Marker ▸ Bind Camera to Markers | Ctrl B | Not available yet |
| Marker ▸ Move Marker | G | Wired |
| Marker ▸ Rename Marker | F2 | Wired |
| Marker ▸ Delete Marker |  | Wired |
| Marker ▸ Duplicate Marker |  | Wired |
| Marker ▸ Add Marker | M | Wired |
| Channel ▸ Delete Channels | X | Not available yet |
| Channel ▸ Group Channels | Ctrl G | Not available yet |
| Channel ▸ Ungroup Channels | Ctrl Alt G | Not available yet |
| Channel ▸ Toggle Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Toggle Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Enable Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Enable Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Disable Channel Setting ▸ Protect |  | Wired |
| Channel ▸ Disable Channel Setting ▸ Mute |  | Wired |
| Channel ▸ Toggle Channel Editability | Tab | Not available yet |
| Channel ▸ Extrapolation Mode ▸ Constant Extrapolation |  | Wired |
| Channel ▸ Extrapolation Mode ▸ Linear Extrapolation |  | Wired |
| Channel ▸ Extrapolation Mode ▸ Make Cyclic (F-Modifier) |  | Not available yet |
| Channel ▸ Extrapolation Mode ▸ Clear Cyclic (F-Modifier) |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Generator |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Built-In Function |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Envelope |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Cycles |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Noise |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Limits |  | Not available yet |
| Channel ▸ Add F-Curve Modifier ▸ Stepped Interpolation |  | Not available yet |
| Channel ▸ Hide Selected Curves | H | Not available yet |
| Channel ▸ Hide Unselected Curves | Shift H | Not available yet |
| Channel ▸ Reveal Curves | Alt H | Not available yet |
| Channel ▸ Expand Channels | Numpad + | Not available yet |
| Channel ▸ Collapse Channels | Numpad - | Not available yet |
| Channel ▸ Move... ▸ To Top | Shift PageUp | Not available yet |
| Channel ▸ Move... ▸ Up | PageUp | Not available yet |
| Channel ▸ Move... ▸ Down | PageDown | Not available yet |
| Channel ▸ Move... ▸ To Bottom | Shift PageDown | Not available yet |
| Channel ▸ Revive Disabled F-Curves |  | Not available yet |
| Channel ▸ Keys to Samples | Alt C | Not available yet |
| Channel ▸ Samples to Keys |  | Not available yet |
| Channel ▸ Sound to Samples |  | Not available yet |
| Channel ▸ Bake Channels |  | Not available yet |
| Channel ▸ Discontinuity (Euler) Filter |  | Not available yet |
| Channel ▸ Frame Selected Channels |  | Not available yet |
| Key ▸ Transform ▸ Move | G | Not available yet |
| Key ▸ Transform ▸ Extend | E | Not available yet |
| Key ▸ Transform ▸ Rotate | R | Not available yet |
| Key ▸ Transform ▸ Scale | S | Not available yet |
| Key ▸ Snap ▸ Selection to Current Frame |  | Wired |
| Key ▸ Snap ▸ Selection to Cursor Value |  | Not available yet |
| Key ▸ Snap ▸ Selection to Nearest Frame |  | Wired |
| Key ▸ Snap ▸ Selection to Nearest Second |  | Wired |
| Key ▸ Snap ▸ Selection to Nearest Marker |  | Wired |
| Key ▸ Snap ▸ Flatten Handles |  | Not available yet |
| Key ▸ Snap ▸ Equalize Handles |  | Not available yet |
| Key ▸ Snap ▸ Cursor to Selection |  | Not available yet |
| Key ▸ Snap ▸ Cursor Value to Selection |  | Not available yet |
| Key ▸ Mirror ▸ By Times Over Current Frame |  | Wired |
| Key ▸ Mirror ▸ By Values Over Cursor Value |  | Not available yet |
| Key ▸ Mirror ▸ By Times Over Zero Time |  | Wired |
| Key ▸ Mirror ▸ By Values Over Zero Value |  | Not available yet |
| Key ▸ Mirror ▸ By Times Over First Selected Marker |  | Wired |
| Key ▸ Jump to Selected | Ctrl G | Not available yet |
| Key ▸ Insert ▸ All Channels |  | Wired |
| Key ▸ Insert ▸ Only Selected Channels |  | Not available yet |
| Key ▸ Insert ▸ Only Active F-Curve |  | Not available yet |
| Key ▸ Insert ▸ Active Channels at Cursor |  | Not available yet |
| Key ▸ Insert ▸ Selected Channels at Cursor |  | Not available yet |
| Key ▸ Copy | Ctrl C | Not available yet |
| Key ▸ Paste | Ctrl V | Not available yet |
| Key ▸ Paste Flipped | Shift Ctrl V | Not available yet |
| Key ▸ Duplicate | Shift D | Wired |
| Key ▸ Delete | X | Not available yet |
| Key ▸ Handle Type ▸ Free |  | Wired |
| Key ▸ Handle Type ▸ Aligned |  | Wired |
| Key ▸ Handle Type ▸ Vector |  | Wired |
| Key ▸ Handle Type ▸ Automatic |  | Wired |
| Key ▸ Handle Type ▸ Auto Clamped |  | Wired |
| Key ▸ Interpolation Mode ▸ Constant |  | Wired |
| Key ▸ Interpolation Mode ▸ Linear |  | Wired |
| Key ▸ Interpolation Mode ▸ Bézier |  | Wired |
| Key ▸ Interpolation Mode ▸ Sinusoidal |  | Wired |
| Key ▸ Interpolation Mode ▸ Quadratic |  | Wired |
| Key ▸ Interpolation Mode ▸ Cubic |  | Wired |
| Key ▸ Interpolation Mode ▸ Quartic |  | Wired |
| Key ▸ Interpolation Mode ▸ Quintic |  | Wired |
| Key ▸ Interpolation Mode ▸ Exponential |  | Wired |
| Key ▸ Interpolation Mode ▸ Circular |  | Wired |
| Key ▸ Interpolation Mode ▸ Back |  | Wired |
| Key ▸ Interpolation Mode ▸ Bounce |  | Wired |
| Key ▸ Interpolation Mode ▸ Elastic |  | Wired |
| Key ▸ Easing Type ▸ Automatic Easing |  | Wired |
| Key ▸ Easing Type ▸ Ease In |  | Wired |
| Key ▸ Easing Type ▸ Ease Out |  | Wired |
| Key ▸ Easing Type ▸ Ease In and Out |  | Wired |
| Key ▸ Density ▸ Decimate (Ratio) |  | Not available yet |
| Key ▸ Density ▸ Decimate (Allowed Change) |  | Not available yet |
| Key ▸ Density ▸ Bake Keyframes | Shift Alt O | Not available yet |
| Key ▸ Density ▸ Clean Keyframes |  | Wired |
| Key ▸ Blend ▸ Breakdown |  | Not available yet |
| Key ▸ Blend ▸ Blend to Neighbor |  | Not available yet |
| Key ▸ Blend ▸ Blend to Default Value |  | Not available yet |
| Key ▸ Blend ▸ Ease |  | Not available yet |
| Key ▸ Blend ▸ Blend Offset |  | Not available yet |
| Key ▸ Blend ▸ Blend to Ease |  | Not available yet |
| Key ▸ Blend ▸ Match Slope |  | Not available yet |
| Key ▸ Blend ▸ Push Pull |  | Not available yet |
| Key ▸ Blend ▸ Shear Keys |  | Not available yet |
| Key ▸ Blend ▸ Scale Average |  | Not available yet |
| Key ▸ Blend ▸ Scale from Neighbor |  | Not available yet |
| Key ▸ Blend ▸ Time Offset |  | Not available yet |
| Key ▸ Smooth ▸ Smooth (Gaussian) |  | Not available yet |
| Key ▸ Smooth ▸ Smooth (Legacy) | Alt O | Not available yet |
| Key ▸ Smooth ▸ Butterworth Smooth |  | Not available yet |

### Preferences

Shown in: a space's switchable area only.

| Menu entry | Shortcut | Status |
|---|---|---|
| View ▸ Sidebar |  | Not available yet |
| View ▸ Toggle Maximize Area | Ctrl Space | Wired |
| Preferences ▸ Auto-Save Preferences |  | Wired |
| Preferences ▸ Save Preferences |  | Not available yet |
| Preferences ▸ Revert to Saved Preferences |  | Not available yet |
| Preferences ▸ Load Factory Preferences |  | Wired |

## Properties

### Active Tool

| Field | Key | Status | Evidence |
|---|---|---|---|
| Tool | `workspace_tool.idname` | Shown only |  |
| Show Gizmo | `show_gizmo` | Read by | `src/scene/overlay.rs` |
| Navigate | `show_gizmo_navigate` | Read by | `src/scene/overlay.rs` |
| Move | `show_gizmo_object_translate` | Shown only |  |
| Rotate | `show_gizmo_object_rotate` | Shown only |  |
| Scale | `show_gizmo_object_scale` | Shown only |  |
| Context | `show_gizmo_context` | Shown only |  |
| Tool | `show_gizmo_tool` | Shown only |  |
| Overlays | `show_overlays` | Read by | `src/scene/overlay.rs` |
| Floor | `show_floor` | Read by | `src/scene/overlay.rs` |
| X Axis | `show_axis_x` | Read by | `src/scene/overlay.rs` |
| Y Axis | `show_axis_y` | Read by | `src/scene/overlay.rs` |
| Z Axis | `show_axis_z` | Read by | `src/scene/overlay.rs` |
| Text Info | `show_text` | Read by | `src/scene/overlay.rs` |
| Statistics | `show_stats` | Read by | `src/scene/overlay.rs` |
| 3D Cursor | `show_cursor` | Read by | `src/scene/overlay.rs` |
| Extras | `show_extras` | Read by | `src/scene/overlay.rs` |
| Relationships | `show_relationship_lines` | Read by | `src/scene/overlay.rs` |
| Outline Selected | `show_outline_selected` | Read by | `src/scene/overlay.rs` |
| Bones | `show_bones` | Read by | `src/scene/overlay.rs` |
| Origins | `show_object_origins` | Read by | `src/scene/overlay.rs` |
| Origins (All) | `show_object_origins_all` | Read by | `src/scene/overlay.rs` |

### Render

| Field | Key | Status | Evidence |
|---|---|---|---|
| Render Engine | `engine` | Rendered |  |
| Device | `cycles.device` | Rendered |  |
| Max Samples | `cycles.preview_samples` | Rendered |  |
| Denoise | `cycles.use_preview_denoising` | Rendered |  |
| Start Sample | `cycles.preview_denoising_start_sample` | Rendered |  |
| Adaptive Sampling | `cycles.use_adaptive_sampling` | Rendered |  |
| Noise Threshold | `cycles.adaptive_threshold` | Rendered |  |
| Max Samples | `cycles.samples` | Rendered |  |
| Min Samples | `cycles.adaptive_min_samples` | Rendered |  |
| Time Limit | `cycles.time_limit` | Rendered |  |
| Denoise | `cycles.use_denoising` | Rendered |  |
| Denoiser | `cycles.denoiser` | n/a | Phiray has one denoiser, its own |
| Total | `cycles.max_bounces` | Rendered |  |
| Diffuse | `cycles.diffuse_bounces` | Rendered |  |
| Glossy | `cycles.glossy_bounces` | Rendered |  |
| Transmission | `cycles.transmission_bounces` | Rendered |  |
| Volume | `cycles.volume_bounces` | n/a | Phiray draws no volumes |
| Transparent | `cycles.transparent_max_bounces` | Rendered |  |
| Direct Light | `cycles.sample_clamp_direct` | Rendered |  |
| Indirect Light | `cycles.sample_clamp_indirect` | Rendered |  |
| Filter Glossy | `cycles.blur_glossy` | Rendered |  |
| Reflective | `cycles.caustics_reflective` | Rendered |  |
| Refractive | `cycles.caustics_refractive` | Rendered |  |
| Step Rate Render | `cycles.volume_step_rate` | n/a | Phiray draws no volumes |
| Step Rate Viewport | `cycles.volume_preview_step_rate` | n/a | Phiray draws no volumes |
| Max Steps | `cycles.volume_max_steps` | n/a | Phiray draws no volumes |
| Motion Blur | `render.use_motion_blur` | Ignored | the renderer does not read it |
| Position | `render.motion_blur_position` | Ignored | the renderer does not read it |
| Shutter | `render.motion_blur_shutter` | Ignored | the renderer does not read it |
| Transparent | `render.film_transparent` | Rendered |  |
| Type | `cycles.pixel_filter_type` | Rendered |  |
| Width | `cycles.filter_width` | Rendered |  |
| Threads Mode | `render.threads_mode` | n/a | a GPU render has no CPU threads |
| Threads | `render.threads` | n/a | a GPU render has no CPU threads |
| Persistent Data | `render.use_persistent_data` | n/a | the scene is always kept between renders |
| Shape | `cycles_curves.shape` | Ignored | the renderer does not read it |
| Curve Subdivisions | `cycles_curves.subdivisions` | Ignored | the renderer does not read it |
| View Transform | `view_settings.view_transform` | Rendered |  |
| Look | `view_settings.look` | Rendered |  |
| Exposure | `view_settings.exposure` | Rendered |  |
| Gamma | `view_settings.gamma` | Rendered |  |

### Output

| Field | Key | Status | Evidence |
|---|---|---|---|
| Resolution X | `resolution_x` | Shown only |  |
| Resolution Y | `resolution_y` | Shown only |  |
| Resolution % | `resolution_percentage` | Shown only |  |
| Render Region | `use_border` | Read by | `src/scene/render_border.rs` |
| Crop to Render Region | `use_crop_to_border` | Read by | `src/scene/render_border.rs` |
| Pixel Aspect X | `pixel_aspect_x` | Shown only |  |
| Pixel Aspect Y | `pixel_aspect_y` | Shown only |  |
| Frame Rate | `fps` | Read by | `src/state/mod.rs` |
| Frame Rate Base | `fps_base` | Shown only |  |
| Frame Start | `frame_start` | Read by | `src/state/mod.rs` |
| End | `frame_end` | Read by | `src/state/mod.rs` |
| Step | `frame_step` | Read by | `src/app/op/render_entry.rs`, `src/state/mod.rs` |
| Stereoscopy | `use_multiview` | Shown only |  |
| Views Format | `views_format` | Shown only |  |
| Output Path | `filepath` | Read by | `src/scene/output_set.rs`, `src/scene/render_edit.rs` |
| Overwrite | `use_overwrite` | Shown only |  |
| Placeholders | `use_placeholder` | Shown only |  |
| File Extensions | `use_file_extension` | Shown only |  |
| Cache Result | `use_render_cache` | Shown only |  |
| File Format | `image_settings.file_format` | Read by | `src/scene/output_set.rs` |
| Color | `image_settings.color_mode` | Read by | `src/scene/output_set.rs` |
| Color Depth | `image_settings.color_depth` | Read by | `src/scene/output_set.rs` |
| Container | `ffmpeg.format` | Read by | `src/scene/output_set.rs` |
| Video Codec | `ffmpeg.codec` | Read by | `src/scene/output_set.rs` |
| Output Quality | `ffmpeg.constant_rate_factor` | Read by | `src/scene/output_set.rs` |
| Encoding Speed | `ffmpeg.ffmpeg_preset` | Read by | `src/scene/output_set.rs` |
| Keyframe Interval | `ffmpeg.gop_size` | Read by | `src/scene/output_set.rs` |
| Bitrate | `ffmpeg.video_bitrate` | Read by | `src/scene/output_set.rs` |
| Include Metadata in Image | `use_stamp` | Shown only |  |
| Time | `use_stamp_time` | Shown only |  |
| Date | `use_stamp_date` | Shown only |  |
| Render Time | `use_stamp_render_time` | Shown only |  |
| Frame | `use_stamp_frame` | Shown only |  |
| Camera | `use_stamp_camera` | Shown only |  |
| Lens | `use_stamp_lens` | Shown only |  |
| Scene | `use_stamp_scene` | Shown only |  |
| Note | `use_stamp_note` | Shown only |  |
| Note Text | `stamp_note_text` | Shown only |  |
| Compositing | `use_compositing` | Shown only |  |
| Sequencer | `use_sequencer` | Shown only |  |
| Dither | `dither_intensity` | Shown only |  |

### View Layer

| Field | Key | Status | Evidence |
|---|---|---|---|
| Name | `name` | Shown only |  |
| Use for Rendering | `use` | Shown only |  |
| Sky | `use_sky` | Shown only |  |
| Solid | `use_solid` | Shown only |  |
| Strand | `use_strand` | Shown only |  |
| Edge Enhance | `use_edge_enhance` | Shown only |  |
| Freestyle | `use_freestyle` | Shown only |  |
| Combined | `use_pass_combined` | Read by | `src/scene/output_set.rs` |
| Z | `use_pass_z` | Read by | `src/scene/output_set.rs` |
| Mist | `use_pass_mist` | Read by | `src/scene/output_set.rs` |
| Normal | `use_pass_normal` | Read by | `src/scene/output_set.rs` |
| Position | `use_pass_position` | Read by | `src/scene/output_set.rs` |
| Vector | `use_pass_vector` | Read by | `src/scene/output_set.rs` |
| UV | `use_pass_uv` | Read by | `src/scene/output_set.rs` |
| Object Index | `use_pass_object_index` | Read by | `src/scene/output_set.rs` |
| Material Index | `use_pass_material_index` | Read by | `src/scene/output_set.rs` |
| Diffuse Direct | `use_pass_diffuse_direct` | Shown only |  |
| Diffuse Indirect | `use_pass_diffuse_indirect` | Shown only |  |
| Diffuse Color | `use_pass_diffuse_color` | Read by | `src/scene/output_set.rs` |
| Glossy Direct | `use_pass_glossy_direct` | Shown only |  |
| Glossy Indirect | `use_pass_glossy_indirect` | Shown only |  |
| Glossy Color | `use_pass_glossy_color` | Shown only |  |
| Emission | `use_pass_emit` | Read by | `src/scene/output_set.rs` |
| Environment | `use_pass_environment` | Read by | `src/scene/output_set.rs` |
| Shadow | `use_pass_shadow` | Shown only |  |
| Ambient Occlusion | `use_pass_ambient_occlusion` | Read by | `src/scene/output_set.rs` |
| Ambient Occlusion | `use_ao` | Shown only |  |
| Solid | `use_solid` | Shown only |  |
| Strand | `use_strand` | Shown only |  |
| Volumes | `use_volumes` | Shown only |  |
| Motion Blur | `use_motion_blur` | Shown only |  |
| Material | `material_override` | Shown only |  |
| World | `world_override` | Shown only |  |
| Samples | `samples` | Read by | `src/scene/render_audit.rs`, `src/scene/render_edit.rs` |

### Scene

| Field | Key | Status | Evidence |
|---|---|---|---|
| Camera | `camera` | Shown only |  |
| Background | `background_set` | Shown only |  |
| Active Clip | `active_clip` | Shown only |  |
| Unit System | `unit_settings.system` | Shown only |  |
| Unit Scale | `unit_settings.scale_length` | Shown only |  |
| Rotation | `unit_settings.system_rotation` | Shown only |  |
| Length | `unit_settings.length_unit` | Shown only |  |
| Mass | `unit_settings.mass_unit` | Shown only |  |
| Time | `unit_settings.time_unit` | Shown only |  |
| Temperature | `unit_settings.temperature_unit` | Shown only |  |
| Active Keying Set | `active_keying_set` | Shown only |  |
| Only Active Keying Set | `use_active_keying_set_only` | Shown only |  |
| Volume | `audio_volume` | Shown only |  |
| Distance Model | `audio_distance_model` | Shown only |  |
| Speed | `audio_doppler_speed` | Shown only |  |
| Doppler | `audio_doppler_factor` | Shown only |  |
| Rigid Body World | `rigidbody_world` | Shown only |  |
| Sub Steps | `rigidbody_world.substeps_per_frame` | Shown only |  |
| Solver Iterations | `rigidbody_world.solver_iterations` | Shown only |  |
| Speed | `rigidbody_world.time_scale` | Shown only |  |
| Split Impulse | `rigidbody_world.use_split_impulse` | Shown only |  |

### World

| Field | Key | Status | Evidence |
|---|---|---|---|
| Use Nodes | `use_nodes` | Shown only |  |
| Node Tree | `node_tree` | Shown only |  |
| Color | `color` | Read by | `src/part/sidebar/node.rs` |
| Volume Shader | `node_tree` | Shown only |  |
| Use Mist | `mist_settings.use_mist` | Shown only |  |
| Start | `mist_settings.start` | Shown only |  |
| Depth | `mist_settings.depth` | Shown only |  |
| Height | `mist_settings.height` | Shown only |  |
| Intensity | `mist_settings.intensity` | Shown only |  |
| Falloff | `mist_settings.falloff` | Shown only |  |
| Color | `color` | Read by | `src/part/sidebar/node.rs` |

### Collection

| Field | Key | Status | Evidence |
|---|---|---|---|
| Hide in Viewport | `hide_viewport` | Shown only |  |
| Disable Selection | `hide_select` | Shown only |  |
| Disable in Renders | `hide_render` | Shown only |  |
| Usage | `lineart_usage` | Shown only |  |
| Intersection Priority | `use_lineart_intersection_mask` | Shown only |  |
| Instance Offset | `instance_offset` | Shown only |  |

### Object

| Field | Key | Status | Evidence |
|---|---|---|---|
| Location | `location` | Applies |  |
| Rotation | `rotation_euler` | Applies |  |
| Mode | `rotation_mode` | Shown only |  |
| Scale | `scale` | Applies |  |
| Dimensions | `dimensions` | Read by | `src/state/key/mod.rs` |
| Location | `delta_location` | Shown only |  |
| Rotation | `delta_rotation_euler` | Shown only |  |
| Scale | `delta_scale` | Shown only |  |
| Parent | `parent` | Shown only |  |
| Parent Type | `parent_type` | Shown only |  |
| Parent Bone | `parent_bone` | Shown only |  |
| Collections | `users_collection` | Shown only |  |
| Instance Type | `instance_type` | Shown only |  |
| Instance Collection | `instance_collection` | Shown only |  |
| Show Instancer for Render | `show_instancer_for_render` | Shown only |  |
| Show Instancer for Viewport | `show_instancer_for_viewport` | Shown only |  |
| Show in Front | `show_in_front` | Shown only |  |
| Disable in Viewports | `hide_viewport` | Shown only |  |
| Disable in Renders | `hide_render` | Shown only |  |
| Disable Selection | `hide_select` | Shown only |  |
| In Front | `show_in_front` | Shown only |  |
| Shadow | `visible_shadow` | Applies |  |
| Display As | `display_type` | Shown only |  |
| Name | `show_name` | Read by | `src/scene/shadow.rs` |
| Axes | `show_axis` | Shown only |  |
| Wireframe | `show_wire` | Shown only |  |
| Bounds | `show_bounds` | Shown only |  |
| Texture Space | `show_texture_space` | Shown only |  |
| Shadow | `show_shadows` | Applies |  |
| Transparent | `show_transparent` | Shown only |  |
| Intersection Priority | `use_lineart_intersection_priority` | Shown only |  |
| Priority | `lineart_intersection_priority` | Shown only |  |

### Modifiers

| Field | Key | Status | Evidence |
|---|---|---|---|
| Add Modifier | `add_modifier` | Shown only |  |
| Modifier Stack | `modifiers` | Shown only |  |

### Particles

| Field | Key | Status | Evidence |
|---|---|---|---|
| Type | `type` | Read by | `src/scene/xform/rule.rs` |
| Number | `count` | Shown only |  |
| Frame Start | `frame_start` | Read by | `src/state/mod.rs` |
| Frame End | `frame_end` | Read by | `src/state/mod.rs` |
| Lifetime | `lifetime` | Shown only |  |
| Lifetime Randomness | `lifetime_random` | Shown only |  |
| Emit From | `emit_from` | Shown only |  |
| Random | `use_emit_random` | Shown only |  |
| Even Distribution | `use_even_distribution` | Shown only |  |
| Physics Type | `physics_type` | Shown only |  |
| Mass | `mass` | Shown only |  |
| Size | `particle_size` | Shown only |  |
| Randomness | `size_random` | Shown only |  |
| Brownian | `brownian_factor` | Shown only |  |
| Damp | `damping` | Shown only |  |
| Render As | `render_type` | Shown only |  |
| Unborn | `show_unborn` | Shown only |  |
| Dead | `use_dead` | Shown only |  |
| Display | `display_method` | Shown only |  |
| Size | `display_size` | Shown only |  |
| Amount | `display_percentage` | Shown only |  |

### Physics

| Field | Key | Status | Evidence |
|---|---|---|---|
| Add Physics | `add_physics` | Shown only |  |
| Type | `rigid_body.type` | Shown only |  |
| Mass | `rigid_body.mass` | Shown only |  |
| Friction | `rigid_body.friction` | Shown only |  |
| Bounciness | `rigid_body.restitution` | Shown only |  |
| Enable Deactivation | `rigid_body.use_deactivation` | Shown only |  |
| Damping | `collision.damping` | Shown only |  |
| Friction Factor | `collision.friction_factor` | Shown only |  |
| Stickiness | `collision.stickiness` | Shown only |  |
| Kill Particles | `collision.use_particle_kill` | Shown only |  |
| Quality Steps | `cloth.settings.quality` | Shown only |  |
| Vertex Mass | `cloth.settings.mass` | Shown only |  |
| Air Viscosity | `cloth.settings.air_damping` | Shown only |  |
| Tension | `cloth.settings.tension_stiffness` | Shown only |  |
| Compression | `cloth.settings.compression_stiffness` | Shown only |  |
| Shear | `cloth.settings.shear_stiffness` | Shown only |  |

### Constraints

| Field | Key | Status | Evidence |
|---|---|---|---|
| Add Object Constraint | `add_constraint` | Shown only |  |
| Constraint Stack | `constraints` | Shown only |  |

### Data

| Field | Key | Status | Evidence |
|---|---|---|---|
| Auto Smooth | `use_auto_smooth` | Shown only |  |
| Angle | `auto_smooth_angle` | Shown only |  |
| UV Maps | `uv_layers` | Shown only |  |
| Vertex Groups | `vertex_groups` | Shown only |  |
| Shape Keys | `shape_keys` | Shown only |  |
| Show Only Shape | `show_only_shape_key` | Shown only |  |
| Attributes | `attributes` | Shown only |  |
| Color Attributes | `color_attributes` | Shown only |  |
| Mode | `remesh_mode` | Shown only |  |
| Voxel Size | `remesh_voxel_size` | Shown only |  |
| Adaptivity | `remesh_voxel_adaptivity` | Shown only |  |
| Geometry Nodes | `modifiers` | Shown only |  |

### Material

| Field | Key | Status | Evidence |
|---|---|---|---|
| Preview Type | `preview_render_type` | Shown only |  |
| Use Nodes | `use_nodes` | Shown only |  |
| Node Tree | `node_tree` | Shown only |  |
| Base Color | `diffuse_color` | Applies |  |
| Image | `base_color_image` | Applies |  |
| Metallic | `metallic` | Applies |  |
| Roughness | `roughness` | Applies |  |
| Specular | `specular_intensity` | Shown only |  |
| Volume Shader | `node_tree` | Shown only |  |
| Backface Culling | `use_backface_culling` | Shown only |  |
| Transparent Shadows | `show_transparent_back` | Shown only |  |
| Blend Mode | `blend_method` | Shown only |  |
| Shadow Mode | `shadow_method` | Shown only |  |
| Alpha Threshold | `alpha_threshold` | Shown only |  |
| Displacement | `displacement_method` | Shown only |  |
| Screen Space Refraction | `use_screen_refraction` | Shown only |  |
| Subsurface Translucency | `use_sss_translucency` | Shown only |  |
| Color | `diffuse_color` | Applies |  |
| Metallic | `metallic` | Applies |  |
| Roughness | `roughness` | Applies |  |

### Texture

| Field | Key | Status | Evidence |
|---|---|---|---|
| Preview Type | `preview_type` | Shown only |  |
| Type | `type` | Read by | `src/scene/xform/rule.rs` |
| Image | `image` | Read by | `src/part/sidebar/set.rs` |
| Extension | `extension` | Shown only |  |
| Interpolation | `interpolation` | Shown only |  |
| Use Alpha | `use_alpha` | Shown only |  |
| Invert Alpha | `invert_alpha` | Shown only |  |
| Use Color Ramp | `use_color_ramp` | Shown only |  |
| Red | `factor_red` | Shown only |  |
| Green | `factor_green` | Shown only |  |
| Blue | `factor_blue` | Shown only |  |
| Intensity | `intensity` | Shown only |  |
| Contrast | `contrast` | Shown only |  |
| Saturation | `saturation` | Shown only |  |
| Projection | `mapping` | Shown only |  |
| Offset | `offset` | Shown only |  |
| Size | `scale` | Applies |  |
| Diffuse Color | `use_map_color_diffuse` | Shown only |  |
| Normal | `use_map_normal` | Shown only |  |
| Alpha | `use_map_alpha` | Shown only |  |
| Specular | `use_map_specular` | Shown only |  |
| Diffuse Factor | `diffuse_color_factor` | Shown only |  |
| Normal Factor | `normal_factor` | Shown only |  |
