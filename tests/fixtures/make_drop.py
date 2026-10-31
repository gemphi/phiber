"""drop.blend: the factory scene's Cube, scaled to half and raised to z = 3,
an active BOX rigid body over a 10 m plane, a passive MESH, in a Rigid
Body World cached over frames 1-60. Prints where Blender's own solver has
the cube at a few frames, the numbers `src/scene/sim/tests.rs`
holds phiber to.

  python make_drop.py drop.blend
"""

import sys

import bpy

bpy.ops.wm.read_factory_settings(use_empty=False)
scene = bpy.context.scene
cube = bpy.data.objects["Cube"]
cube.location = (0.0, 0.0, 3.0)
cube.scale = (0.5, 0.5, 0.5)
bpy.ops.mesh.primitive_plane_add(size=10.0, location=(0.0, 0.0, 0.0))
plane = bpy.context.active_object
plane.name = "Ground"


def rigid(ob, kind, shape):
    bpy.context.view_layer.objects.active = ob
    for o in bpy.context.view_layer.objects:
        o.select_set(o == ob)
    bpy.ops.rigidbody.object_add(type=kind)
    ob.rigid_body.collision_shape = shape


rigid(cube, "ACTIVE", "BOX")
rigid(plane, "PASSIVE", "MESH")
scene.rigidbody_world.point_cache.frame_start = 1
scene.rigidbody_world.point_cache.frame_end = 60
scene.frame_start, scene.frame_end = 1, 60
# The world steps only frame by frame, from the cache's first frame.
for f in range(1, 61):
    scene.frame_set(f)
    if f in (1, 10, 20, 30, 40, 60):
        z = cube.evaluated_get(bpy.context.evaluated_depsgraph_get()).matrix_world.translation.z
        print(f"frame {f}: z = {z:.4f}")
scene.frame_set(1)
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
