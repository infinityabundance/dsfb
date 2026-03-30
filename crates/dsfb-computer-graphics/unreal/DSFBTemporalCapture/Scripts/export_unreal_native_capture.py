import json
import shutil
from pathlib import Path

import unreal


WIDTH = 256
HEIGHT = 144
REFERENCE_SCALE = 4
REFERENCE_WIDTH = WIDTH * REFERENCE_SCALE
REFERENCE_HEIGHT = HEIGHT * REFERENCE_SCALE
RELATIVE_OUTPUT_ROOT = "../../data/unreal_native/sample_capture"
RAW_DIR_NAME = "raw"
SHOT_NAME = "minimal_temporal_sequence"

PLANE_LOCATION = unreal.Vector(300.0, 0.0, 0.0)
PLANE_SCALE = unreal.Vector(40.0, 40.0, 1.0)
CUBE_SCALE = unreal.Vector(2.5, 2.5, 2.5)
CAMERA_LOCATION = unreal.Vector(-600.0, 0.0, 140.0)
CAMERA_ROTATION = unreal.Rotator(-6.0, 0.0, 0.0)
LIGHT_LOCATION = unreal.Vector(0.0, 0.0, 220.0)
LIGHT_ROTATION = unreal.Rotator(-40.0, 20.0, 0.0)

FRAME_SETTLE_TICKS = 40
MOVE_SETTLE_TICKS = 40

SEQUENCE_CENTERS = [
    unreal.Vector(0.0, -220.0, 80.0),
    unreal.Vector(0.0, -80.0, 80.0),
    unreal.Vector(0.0, 220.0, 80.0),
    unreal.Vector(0.0, 120.0, 80.0),
    unreal.Vector(0.0, 40.0, 80.0),
    unreal.Vector(0.0, -140.0, 80.0),
]

FRAME_SPECS = [
    {
        "label": f"frame_{index:04d}",
        "frame_index": index,
        "history_frame_index": index - 1,
        "previous_center": SEQUENCE_CENTERS[index - 1],
        "current_center": SEQUENCE_CENTERS[index],
    }
    for index in range(1, len(SEQUENCE_CENTERS))
]

CAPTURE_SPECS = {
    "previous_color": {
        "phase": "previous",
        "file_name": "previous_color.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": None,
        "semantic": "final_color_png",
        "width": WIDTH,
        "height": HEIGHT,
    },
    "current_color": {
        "phase": "current",
        "file_name": "current_color.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": None,
        "semantic": "final_color_png",
        "width": WIDTH,
        "height": HEIGHT,
    },
    "reference_color_hi": {
        "phase": "current",
        "file_name": "reference_color_hi.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": None,
        "semantic": "final_color_supersampled_proxy_png",
        "width": REFERENCE_WIDTH,
        "height": REFERENCE_HEIGHT,
    },
    "previous_normals": {
        "phase": "previous",
        "file_name": "previous_normals.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/WorldNormal.WorldNormal",
        "semantic": "world_normal_visualization_png",
        "width": WIDTH,
        "height": HEIGHT,
    },
    "current_normals": {
        "phase": "current",
        "file_name": "current_normals.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/WorldNormal.WorldNormal",
        "semantic": "world_normal_visualization_png",
        "width": WIDTH,
        "height": HEIGHT,
    },
    "previous_depth": {
        "phase": "previous",
        "file_name": "previous_depth.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/SceneDepth.SceneDepth",
        "semantic": "scene_depth_visualization_png",
        "width": WIDTH,
        "height": HEIGHT,
    },
    "current_depth": {
        "phase": "current",
        "file_name": "current_depth.png",
        "capture_source": unreal.SceneCaptureSource.SCS_FINAL_COLOR_LDR,
        "material_path": "/Engine/BufferVisualization/SceneDepth.SceneDepth",
        "semantic": "scene_depth_visualization_png",
        "width": WIDTH,
        "height": HEIGHT,
    },
}


def project_dir() -> Path:
    return Path(unreal.Paths.project_dir()).resolve()


def output_root_dir() -> Path:
    return (project_dir() / RELATIVE_OUTPUT_ROOT).resolve()


def frame_output_dir(frame_label: str) -> Path:
    return output_root_dir() / frame_label


def raw_output_dir(frame_label: str) -> Path:
    return frame_output_dir(frame_label) / RAW_DIR_NAME


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


def create_render_target(width: int, height: int) -> unreal.TextureRenderTarget2D:
    return unreal.RenderingLibrary.create_render_target2d(
        world(),
        width,
        height,
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
    component.set_editor_property(
        "texture_target",
        create_render_target(int(spec["width"]), int(spec["height"])),
    )
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
    root = output_root_dir()
    if root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True, exist_ok=True)
    for frame in FRAME_SPECS:
        raw_output_dir(frame["label"]).mkdir(parents=True, exist_ok=True)


def move_actor(actor, location: unreal.Vector) -> None:
    actor.set_actor_location(location, False, False)
    invalidate_viewports()


def ensure_scene(initial_cube_center: unreal.Vector):
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
        initial_cube_center,
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


def export_capture(component, frame_label: str, file_name: str) -> None:
    component.capture_scene()
    invalidate_viewports()
    unreal.RenderingLibrary.export_render_target(
        world(),
        component.get_editor_property("texture_target"),
        str(raw_output_dir(frame_label)),
        file_name,
    )


def export_phase(frame_label: str, phase: str, devices: dict[str, dict[str, object]]) -> None:
    for device in devices.values():
        spec = device["spec"]
        if spec["phase"] != phase:
            continue
        export_capture(device["component"], frame_label, spec["file_name"])


def validate_exports(frame_label: str) -> None:
    missing = [
        spec["file_name"]
        for spec in CAPTURE_SPECS.values()
        if not (raw_output_dir(frame_label) / spec["file_name"]).exists()
    ]
    if missing:
        raise RuntimeError(
            f"Unreal export for {frame_label} did not produce the required raw files: {', '.join(missing)}"
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


def write_scene_state(frame: dict[str, object], camera) -> None:
    state = {
        "schema_version": "dsfb_unreal_capture_scene_state_v2",
        "dataset_kind": "unreal_native",
        "provenance_label": "unreal_native",
        "frame_label": frame["label"],
        "frame_index": frame["frame_index"],
        "history_frame_index": frame["history_frame_index"],
        "width": WIDTH,
        "height": HEIGHT,
        "shot_name": SHOT_NAME,
        "sequence_length": len(FRAME_SPECS),
        "reference_scale": REFERENCE_SCALE,
        "camera": camera_metadata(camera),
        "scene": {
            "plane": {
                "center": vector_to_list(PLANE_LOCATION),
                "scale": vector_to_list(PLANE_SCALE),
                "nominal_mesh_extent_xy": [50.0, 50.0],
                "normal": [0.0, 0.0, 1.0],
            },
            "cube": {
                "previous_center": vector_to_list(frame["previous_center"]),
                "current_center": vector_to_list(frame["current_center"]),
                "scale": vector_to_list(CUBE_SCALE),
                "nominal_half_extent": [50.0, 50.0, 50.0],
            },
            "sequence_centers": [vector_to_list(center) for center in SEQUENCE_CENTERS],
        },
        "raw_exports": {
            label: {
                "path": f"{RAW_DIR_NAME}/{spec['file_name']}",
                "semantic": spec["semantic"],
                "width": int(spec["width"]),
                "height": int(spec["height"]),
            }
            for label, spec in CAPTURE_SPECS.items()
        },
        "notes": [
            "This file records the exact Unreal-side camera and object transforms used for the real export sequence.",
            "The crate-local postprocess builder converts these raw Unreal PNG exports into the strict DSFB replay dataset.",
            "reference_color_hi.png is a real Unreal final-color export rendered at a higher resolution for downsampled reference-proxy construction.",
            "If raw float velocity export is unavailable on this editor path, motion_vectors.json is derived from Unreal camera/object metadata rather than fabricated from a synthetic scene.",
        ],
    }
    (frame_output_dir(frame["label"]) / "scene_state.json").write_text(
        json.dumps(state, indent=2),
        encoding="utf-8",
    )


def write_capture_log(frame: dict[str, object]) -> None:
    contents = f"""# DSFB Unreal-Native Raw Export Log

This directory contains the raw Unreal exports for {frame["label"]}.

Raw export directory:
- {raw_output_dir(frame["label"])}

Resolution:
- canonical replay inputs: {WIDTH}x{HEIGHT}
- reference proxy export: {REFERENCE_WIDTH}x{REFERENCE_HEIGHT}

Frame mapping:
- frame_index = {frame["frame_index"]}
- history_frame_index = {frame["history_frame_index"]}
- shot_name = {SHOT_NAME}

Raw files produced:
- raw/previous_color.png
- raw/current_color.png
- raw/reference_color_hi.png
- raw/previous_normals.png
- raw/current_normals.png
- raw/previous_depth.png
- raw/current_depth.png
- scene_state.json

Capture method:
- SceneCapture2D with SCS_FINAL_COLOR_LDR for color
- SceneCapture2D plus /Engine/BufferVisualization/WorldNormal.WorldNormal for normals
- SceneCapture2D plus /Engine/BufferVisualization/SceneDepth.SceneDepth for depth
- higher-resolution SceneCapture2D final-color export for the reference proxy

Important notes:
- This stage exports only real Unreal-originated images and scene metadata.
- The crate-local postprocess builder materializes the strict replay JSON buffers, derived motion vectors, and downsampled reference proxy.
- Unreal-native mode still refuses synthetic or proxy provenance.
"""
    (frame_output_dir(frame["label"]) / "capture_commands.txt").write_text(
        contents,
        encoding="utf-8",
    )


class CaptureDriver:
    def __init__(self) -> None:
        ensure_output_dirs()
        first_frame = FRAME_SPECS[0]
        self.frame_cursor = 0
        self.cube, self.camera = ensure_scene(first_frame["previous_center"])
        self.devices = build_capture_devices(self.camera)
        self.phase = "settle_previous"
        self.wait_ticks = FRAME_SETTLE_TICKS
        self.handle = unreal.register_slate_post_tick_callback(self.tick)

    def current_frame(self) -> dict[str, object]:
        return FRAME_SPECS[self.frame_cursor]

    def shutdown(self) -> None:
        if self.handle is not None:
            unreal.unregister_slate_post_tick_callback(self.handle)
            self.handle = None

    def fail(self, message: str) -> None:
        unreal.log_error(message)
        self.shutdown()
        unreal.SystemLibrary.quit_editor()

    def finish(self) -> None:
        self.shutdown()
        unreal.log(f"DSFB Unreal raw export finished at {output_root_dir()}")
        unreal.SystemLibrary.quit_editor()

    def complete_frame(self) -> None:
        frame = self.current_frame()
        validate_exports(frame["label"])
        write_scene_state(frame, self.camera)
        write_capture_log(frame)
        if self.frame_cursor + 1 >= len(FRAME_SPECS):
            self.finish()
            return
        self.frame_cursor += 1
        next_frame = self.current_frame()
        move_actor(self.cube, next_frame["previous_center"])
        self.phase = "settle_previous"
        self.wait_ticks = MOVE_SETTLE_TICKS

    def tick(self, _delta_time: float) -> None:
        try:
            if self.wait_ticks > 0:
                self.wait_ticks -= 1
                return

            frame = self.current_frame()
            if self.phase == "settle_previous":
                export_phase(frame["label"], "previous", self.devices)
                self.phase = "move_cube"
                self.wait_ticks = MOVE_SETTLE_TICKS
                return

            if self.phase == "move_cube":
                move_actor(self.cube, frame["current_center"])
                self.phase = "settle_current"
                self.wait_ticks = MOVE_SETTLE_TICKS
                return

            if self.phase == "settle_current":
                export_phase(frame["label"], "current", self.devices)
                self.phase = "complete_frame"
                return

            if self.phase == "complete_frame":
                self.complete_frame()
        except Exception as exc:  # pragma: no cover - Unreal-only path
            self.fail(f"DSFB Unreal raw export failed: {exc}")


def main() -> None:
    unreal.EditorPythonScripting.set_keep_python_script_alive(True)
    CaptureDriver()
    unreal.log(f"DSFB Unreal raw export starting in {output_root_dir()}")


if __name__ == "__main__":
    main()
