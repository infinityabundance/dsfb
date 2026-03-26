import json
import shutil
from pathlib import Path

import unreal


WIDTH = 256
HEIGHT = 144
RELATIVE_OUTPUT_ROOT = "../../data/unreal_native/sample_capture/frame_0001"
RAW_DIR_NAME = "raw"

PLANE_LOCATION = unreal.Vector(300.0, 0.0, 0.0)
PLANE_SCALE = unreal.Vector(40.0, 40.0, 1.0)
CUBE_PREVIOUS_LOCATION = unreal.Vector(0.0, -220.0, 80.0)
CUBE_CURRENT_LOCATION = unreal.Vector(0.0, 220.0, 80.0)
CUBE_SCALE = unreal.Vector(2.5, 2.5, 2.5)
CAMERA_LOCATION = unreal.Vector(-600.0, 0.0, 140.0)
CAMERA_ROTATION = unreal.Rotator(-6.0, 0.0, 0.0)
LIGHT_LOCATION = unreal.Vector(0.0, 0.0, 220.0)
LIGHT_ROTATION = unreal.Rotator(-40.0, 20.0, 0.0)

FRAME_SETTLE_TICKS = 40
MOVE_SETTLE_TICKS = 40

CAPTURE_SPECS = {
    "previous_color": {
        "phase": "previous",
        "file_name": "previous_color.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": None,
        "semantic": "final_color_png",
    },
    "current_color": {
        "phase": "current",
        "file_name": "current_color.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": None,
        "semantic": "final_color_png",
    },
    "previous_normals": {
        "phase": "previous",
        "file_name": "previous_normals.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/WorldNormal.WorldNormal",
        "semantic": "world_normal_visualization_png",
    },
    "current_normals": {
        "phase": "current",
        "file_name": "current_normals.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/WorldNormal.WorldNormal",
        "semantic": "world_normal_visualization_png",
    },
    "previous_depth": {
        "phase": "previous",
        "file_name": "previous_depth.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/SceneDepth.SceneDepth",
        "semantic": "scene_depth_visualization_png",
    },
    "current_depth": {
        "phase": "current",
        "file_name": "current_depth.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/SceneDepth.SceneDepth",
        "semantic": "scene_depth_visualization_png",
    },
}


def project_dir() -> Path:
    return Path(unreal.Paths.project_dir()).resolve()


def output_dir() -> Path:
    return (project_dir() / RELATIVE_OUTPUT_ROOT).resolve()


def raw_output_dir() -> Path:
    return output_dir() / RAW_DIR_NAME


def world():
    return unreal.get_editor_subsystem(unreal.UnrealEditorSubsystem).get_editor_world()


def level_editor_subsystem():
    return unreal.get_editor_subsystem(unreal.LevelEditorSubsystem)


def invalidate_viewports() -> None:
    level_editor_subsystem().editor_invalidate_viewports()


def delete_existing_scene() -> None:
    prefixes = ("DSFBTemporalCapture_",)
    for actor in list(unreal.EditorLevelLibrary.get_all_level_actors()):
        if actor.get_actor_label().startswith(prefixes):
            unreal.EditorLevelLibrary.destroy_actor(actor)


def create_render_target() -> unreal.TextureRenderTarget2D:
    return unreal.RenderingLibrary.create_render_target2d(
        world(),
        WIDTH,
        HEIGHT,
        unreal.TextureRenderTargetFormat.RTF_RGBA8_SRGB,
        unreal.LinearColor(0.0, 0.0, 0.0, 1.0),
        False,
        False,
    )


def spawn_capture(camera, label: str, spec: dict[str, object]):
    actor = unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.SceneCapture2D,
        camera.get_actor_location(),
        camera.get_actor_rotation(),
    )
    actor.set_actor_label(label)
    component = actor.get_component_by_class(unreal.SceneCaptureComponent2D)
    if component is None:
        raise RuntimeError(f"Unable to resolve SceneCaptureComponent2D for {label}")
    component.set_editor_property("capture_every_frame", False)
    component.set_editor_property("capture_on_movement", False)
    component.set_editor_property("always_persist_rendering_state", True)
    component.set_editor_property("capture_source", spec["capture_source"])
    component.set_editor_property("texture_target", create_render_target())
    component.set_editor_property(
        "fov_angle",
        camera.get_cine_camera_component().get_editor_property("field_of_view"),
    )
    material_path = spec["material_path"]
    if material_path:
        material = unreal.EditorAssetLibrary.load_asset(material_path)
        if material is None:
            raise RuntimeError(f"Unable to load capture material {material_path}")
        component.add_or_update_blendable(material, 1.0)
    return component


def ensure_output_dirs() -> None:
    if raw_output_dir().exists():
        shutil.rmtree(raw_output_dir())
    raw_output_dir().mkdir(parents=True, exist_ok=True)
    for file_name in [
        "current_color.exr",
        "current_color.json",
        "previous_color.exr",
        "previous_color.json",
        "current_depth.exr",
        "current_depth.json",
        "previous_depth.exr",
        "previous_depth.json",
        "current_normals.exr",
        "current_normals.json",
        "previous_normals.exr",
        "previous_normals.json",
        "motion_vectors.exr",
        "motion_vectors.json",
        "host_output.exr",
        "roi_mask.json",
        "disocclusion_mask.json",
        "metadata.json",
        "scene_state.json",
        "capture_commands.txt",
    ]:
        path = output_dir() / file_name
        if path.exists():
            path.unlink()


def move_actor(actor, location: unreal.Vector) -> None:
    actor.set_actor_location(location, False, False)
    invalidate_viewports()


def ensure_scene():
    delete_existing_scene()

    cube_asset = unreal.EditorAssetLibrary.load_asset("/Engine/BasicShapes/Cube.Cube")
    plane_asset = unreal.EditorAssetLibrary.load_asset("/Engine/BasicShapes/Plane.Plane")
    if cube_asset is None or plane_asset is None:
        raise RuntimeError("Unable to load Unreal basic shape assets")

    plane = unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.StaticMeshActor,
        PLANE_LOCATION,
        unreal.Rotator(0.0, 0.0, 0.0),
    )
    plane.set_actor_label("DSFBTemporalCapture_Plane")
    plane.static_mesh_component.set_static_mesh(plane_asset)
    plane.set_actor_scale3d(PLANE_SCALE)

    cube = unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.StaticMeshActor,
        CUBE_PREVIOUS_LOCATION,
        unreal.Rotator(0.0, 0.0, 0.0),
    )
    cube.set_actor_label("DSFBTemporalCapture_Cube")
    cube.static_mesh_component.set_static_mesh(cube_asset)
    cube.set_actor_scale3d(CUBE_SCALE)

    camera = unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.CineCameraActor,
        CAMERA_LOCATION,
        CAMERA_ROTATION,
    )
    camera.set_actor_label("DSFBTemporalCapture_Camera")
    camera_component = camera.get_cine_camera_component()
    camera_component.set_editor_property("current_focal_length", 28.0)

    light = unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.DirectionalLight,
        LIGHT_LOCATION,
        LIGHT_ROTATION,
    )
    light.set_actor_label("DSFBTemporalCapture_Light")

    sky = unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.SkyLight,
        unreal.Vector(0.0, 0.0, 140.0),
        unreal.Rotator(0.0, 0.0, 0.0),
    )
    sky.set_actor_label("DSFBTemporalCapture_Skylight")

    unreal.EditorLevelLibrary.pilot_level_actor(camera)
    invalidate_viewports()
    return cube, camera


def build_capture_devices(camera) -> dict[str, dict[str, object]]:
    devices = {}
    for label, spec in CAPTURE_SPECS.items():
        devices[label] = {
            "component": spawn_capture(
                camera,
                f"DSFBTemporalCaptureCapture_{label}",
                spec,
            ),
            "spec": spec,
        }
    return devices


def export_capture(component, file_name: str) -> None:
    component.capture_scene()
    invalidate_viewports()
    unreal.RenderingLibrary.export_render_target(
        world(),
        component.get_editor_property("texture_target"),
        str(raw_output_dir()),
        file_name,
    )


def export_phase(phase: str, devices: dict[str, dict[str, object]]) -> None:
    for label, device in devices.items():
        spec = device["spec"]
        if spec["phase"] != phase:
            continue
        export_capture(device["component"], spec["file_name"])


def validate_exports() -> None:
    missing = [
        spec["file_name"]
        for spec in CAPTURE_SPECS.values()
        if not (raw_output_dir() / spec["file_name"]).exists()
    ]
    if missing:
        raise RuntimeError(
            f"Unreal export did not produce the required raw files: {', '.join(missing)}"
        )


def vector_to_list(value: unreal.Vector) -> list[float]:
    return [float(value.x), float(value.y), float(value.z)]


def camera_metadata(camera) -> dict[str, object]:
    component = camera.get_cine_camera_component()
    return {
        "name": camera.get_actor_label(),
        "position": vector_to_list(camera.get_actor_location()),
        "forward": vector_to_list(camera.get_actor_forward_vector()),
        "right": vector_to_list(camera.get_actor_right_vector()),
        "up": vector_to_list(camera.get_actor_up_vector()),
        "rotation": [
            float(camera.get_actor_rotation().pitch),
            float(camera.get_actor_rotation().yaw),
            float(camera.get_actor_rotation().roll),
        ],
        "fov_degrees": float(component.get_editor_property("field_of_view")),
        "focal_length_mm": float(component.get_editor_property("current_focal_length")),
    }


def write_scene_state(camera) -> None:
    state = {
        "schema_version": "dsfb_unreal_capture_scene_state_v1",
        "dataset_kind": "unreal_native",
        "provenance_label": "unreal_native",
        "frame_label": "frame_0001",
        "width": WIDTH,
        "height": HEIGHT,
        "camera": camera_metadata(camera),
        "scene": {
            "plane": {
                "center": vector_to_list(PLANE_LOCATION),
                "scale": vector_to_list(PLANE_SCALE),
                "nominal_mesh_extent_xy": [50.0, 50.0],
                "normal": [0.0, 0.0, 1.0],
            },
            "cube": {
                "previous_center": vector_to_list(CUBE_PREVIOUS_LOCATION),
                "current_center": vector_to_list(CUBE_CURRENT_LOCATION),
                "scale": vector_to_list(CUBE_SCALE),
                "nominal_half_extent": [50.0, 50.0, 50.0],
            },
        },
        "raw_exports": {
            label: {
                "path": f"{RAW_DIR_NAME}/{spec['file_name']}",
                "semantic": spec["semantic"],
            }
            for label, spec in CAPTURE_SPECS.items()
        },
        "notes": [
            "This file records the exact Unreal-side camera and object transforms used for the real export.",
            "The crate-local postprocess builder converts these raw Unreal PNG exports into the strict DSFB replay dataset.",
            "If raw float velocity export is unavailable on this editor path, motion_vectors.json is derived from Unreal camera/object metadata rather than fabricated from a synthetic scene.",
        ],
    }
    (output_dir() / "scene_state.json").write_text(
        json.dumps(state, indent=2),
        encoding="utf-8",
    )


def write_capture_log() -> None:
    contents = f"""# DSFB Unreal-Native Raw Export Log

This directory contains the raw Unreal exports for the checked-in minimal sample.

Raw export directory:
- {raw_output_dir()}

Resolution:
- {WIDTH}x{HEIGHT}

Raw files produced:
- raw/previous_color.png
- raw/current_color.png
- raw/previous_normals.png
- raw/current_normals.png
- raw/previous_depth.png
- raw/current_depth.png
- scene_state.json

Capture method:
- SceneCapture2D with SCS_FINAL_COLOR_LDR for color
- SceneCapture2D plus /Engine/BufferVisualization/WorldNormal.WorldNormal for normals
- SceneCapture2D plus /Engine/BufferVisualization/SceneDepth.SceneDepth for depth

Important notes:
- This stage exports only real Unreal-originated images and scene metadata.
- The crate-local postprocess builder materializes the strict replay JSON buffers and derived motion vectors.
- Unreal-native mode still refuses synthetic or proxy provenance.
"""
    (output_dir() / "capture_commands.txt").write_text(contents, encoding="utf-8")


class CaptureDriver:
    def __init__(self) -> None:
        ensure_output_dirs()
        self.cube, self.camera = ensure_scene()
        self.devices = build_capture_devices(self.camera)
        self.phase = "settle_previous"
        self.wait_ticks = FRAME_SETTLE_TICKS
        self.handle = unreal.register_slate_post_tick_callback(self.tick)

    def shutdown(self) -> None:
        if self.handle is not None:
            unreal.unregister_slate_post_tick_callback(self.handle)
            self.handle = None

    def fail(self, message: str) -> None:
        unreal.log_error(message)
        self.shutdown()
        unreal.SystemLibrary.quit_editor()

    def finish(self) -> None:
        validate_exports()
        write_scene_state(self.camera)
        write_capture_log()
        self.shutdown()
        unreal.log(f"DSFB Unreal raw export finished at {output_dir()}")
        unreal.SystemLibrary.quit_editor()

    def tick(self, _delta_time: float) -> None:
        try:
            if self.wait_ticks > 0:
                self.wait_ticks -= 1
                return

            if self.phase == "settle_previous":
                export_phase("previous", self.devices)
                self.phase = "move_cube"
                self.wait_ticks = MOVE_SETTLE_TICKS
                return

            if self.phase == "move_cube":
                move_actor(self.cube, CUBE_CURRENT_LOCATION)
                self.phase = "settle_current"
                self.wait_ticks = MOVE_SETTLE_TICKS
                return

            if self.phase == "settle_current":
                export_phase("current", self.devices)
                self.phase = "complete"
                return

            if self.phase == "complete":
                self.finish()
        except Exception as exc:  # pragma: no cover - Unreal-only path
            self.fail(f"DSFB Unreal raw export failed: {exc}")


def main() -> None:
    unreal.EditorPythonScripting.set_keep_python_script_alive(True)
    CaptureDriver()
    unreal.log(f"DSFB Unreal raw export starting in {output_dir()}")


if __name__ == "__main__":
    main()
