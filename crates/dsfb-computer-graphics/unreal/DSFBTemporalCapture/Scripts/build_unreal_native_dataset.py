import json
import math
from pathlib import Path

from PIL import Image


FRAME_LABEL = "frame_0001"


def crate_root() -> Path:
    return Path(__file__).resolve().parents[3]


def frame_dir() -> Path:
    return crate_root() / "data" / "unreal_native" / "sample_capture" / FRAME_LABEL


def raw_dir() -> Path:
    return frame_dir() / "raw"


def manifest_path() -> Path:
    return crate_root() / "examples" / "unreal_native_capture_manifest.json"


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload, pretty: bool = False) -> None:
    path.write_text(
        json.dumps(
            payload,
            indent=2 if pretty else None,
            separators=None if pretty else (",", ":"),
        ),
        encoding="utf-8",
    )


def srgb_to_linear(value: float) -> float:
    if value <= 0.04045:
        return value / 12.92
    return ((value + 0.055) / 1.055) ** 2.4


def normalize(vec: tuple[float, float, float]) -> tuple[float, float, float]:
    length = math.sqrt(vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2])
    if length <= 1.0e-8:
        return (0.0, 0.0, 1.0)
    return (vec[0] / length, vec[1] / length, vec[2] / length)


def vec_add(left, right):
    return (left[0] + right[0], left[1] + right[1], left[2] + right[2])


def vec_sub(left, right):
    return (left[0] - right[0], left[1] - right[1], left[2] - right[2])


def vec_mul(value, scalar: float):
    return (value[0] * scalar, value[1] * scalar, value[2] * scalar)


def dot(left, right) -> float:
    return left[0] * right[0] + left[1] * right[1] + left[2] * right[2]


def rgb_tuples(path: Path, expected_width: int, expected_height: int) -> list[tuple[int, int, int]]:
    image = Image.open(path).convert("RGB")
    if image.size != (expected_width, expected_height):
        raise RuntimeError(
            f"{path} had extent {image.size[0]}x{image.size[1]} but {expected_width}x{expected_height} was required"
        )
    return [image.getpixel((x, y)) for y in range(expected_height) for x in range(expected_width)]


def linearize_color(rgb_pixels: list[tuple[int, int, int]]) -> list[list[float]]:
    return [
        [srgb_to_linear(channel / 255.0) for channel in (r, g, b)]
        for r, g, b in rgb_pixels
    ]


def decode_depth(rgb_pixels: list[tuple[int, int, int]]) -> list[float]:
    return [((r + g + b) / 3.0) / 255.0 for r, g, b in rgb_pixels]


def current_vs_previous_difference(
    current: list[list[float]], previous: list[list[float]]
) -> float:
    return max(
        max(
            abs(curr[channel] - prev[channel])
            for channel in range(3)
        )
        for curr, prev in zip(current, previous)
    )


def current_vs_previous_diff_count(
    current: list[list[float]], previous: list[list[float]], threshold: float
) -> int:
    return sum(
        1
        for curr, prev in zip(current, previous)
        if max(abs(curr[channel] - prev[channel]) for channel in range(3)) > threshold
    )


def validate_raw_exports(
    width: int,
    height: int,
    current_color: list[list[float]],
    previous_color: list[list[float]],
    current_depth: list[float],
    current_normals_rgb: list[tuple[int, int, int]],
    previous_normals_rgb: list[tuple[int, int, int]],
) -> None:
    max_color = max(max(pixel) for pixel in current_color)
    if max_color <= 1.0e-4:
        raise RuntimeError("current_color.png decoded to a black frame")

    diff_max = current_vs_previous_difference(current_color, previous_color)
    diff_count = current_vs_previous_diff_count(current_color, previous_color, 0.05)
    if diff_max <= 0.05 or diff_count <= (width * height) // 32:
        raise RuntimeError(
            "current_color.png and previous_color.png did not show meaningful temporal motion"
        )

    if max(current_depth) - min(current_depth) <= 0.02:
        raise RuntimeError("current_depth.png did not contain usable depth variation")

    normal_signal = max(max(pixel) for pixel in current_normals_rgb)
    normal_diff = sum(
        1 for current, previous in zip(current_normals_rgb, previous_normals_rgb) if current != previous
    )
    if normal_signal <= 16 or normal_diff <= (width * height) // 64:
        raise RuntimeError(
            "current_normals.png did not contain a meaningful Unreal normal-visualization signal"
        )


def camera_ray(camera: dict[str, object], x: int, y: int, width: int, height: int):
    aspect = width / float(height)
    tan_half_fov = math.tan(math.radians(camera["fov_degrees"]) * 0.5)
    ndc_x = ((x + 0.5) / float(width)) * 2.0 - 1.0
    ndc_y = 1.0 - ((y + 0.5) / float(height)) * 2.0
    forward = tuple(camera["forward"])
    right = tuple(camera["right"])
    up = tuple(camera["up"])
    direction = normalize(
        vec_add(
            forward,
            vec_add(
                vec_mul(right, ndc_x * aspect * tan_half_fov),
                vec_mul(up, ndc_y * tan_half_fov),
            ),
        )
    )
    return tuple(camera["position"]), direction


def intersect_plane(origin, direction, plane_center, plane_half_extent_xy):
    if abs(direction[2]) <= 1.0e-6:
        return None
    t = (plane_center[2] - origin[2]) / direction[2]
    if t <= 0.0:
        return None
    point = vec_add(origin, vec_mul(direction, t))
    if abs(point[0] - plane_center[0]) > plane_half_extent_xy[0]:
        return None
    if abs(point[1] - plane_center[1]) > plane_half_extent_xy[1]:
        return None
    return {
        "kind": "plane",
        "point": point,
        "depth": t,
        "normal": (0.0, 0.0, 1.0),
    }


def intersect_aabb(origin, direction, center, half_extent):
    bounds_min = (
        center[0] - half_extent[0],
        center[1] - half_extent[1],
        center[2] - half_extent[2],
    )
    bounds_max = (
        center[0] + half_extent[0],
        center[1] + half_extent[1],
        center[2] + half_extent[2],
    )
    t_min = -float("inf")
    t_max = float("inf")
    enter_normal = None

    for axis in range(3):
        origin_value = origin[axis]
        direction_value = direction[axis]
        if abs(direction_value) <= 1.0e-8:
            if origin_value < bounds_min[axis] or origin_value > bounds_max[axis]:
                return None
            continue

        inv = 1.0 / direction_value
        t1 = (bounds_min[axis] - origin_value) * inv
        t2 = (bounds_max[axis] - origin_value) * inv
        n1 = [0.0, 0.0, 0.0]
        n2 = [0.0, 0.0, 0.0]
        n1[axis] = -1.0
        n2[axis] = 1.0
        if t1 > t2:
            t1, t2 = t2, t1
            n1, n2 = n2, n1
        if t1 > t_min:
            t_min = t1
            enter_normal = tuple(n1)
        if t2 < t_max:
            t_max = t2
        if t_min > t_max:
            return None

    if t_max <= 0.0:
        return None
    depth = t_min if t_min > 0.0 else t_max
    point = vec_add(origin, vec_mul(direction, depth))
    return {
        "kind": "cube",
        "point": point,
        "depth": depth,
        "normal": enter_normal if enter_normal is not None else (0.0, 0.0, 1.0),
    }


def project_point(camera: dict[str, object], point, width: int, height: int):
    relative = vec_sub(point, tuple(camera["position"]))
    camera_x = dot(relative, tuple(camera["right"]))
    camera_y = dot(relative, tuple(camera["up"]))
    camera_z = dot(relative, tuple(camera["forward"]))
    if camera_z <= 1.0e-6:
        return None
    aspect = width / float(height)
    tan_half_fov = math.tan(math.radians(camera["fov_degrees"]) * 0.5)
    ndc_x = camera_x / (camera_z * tan_half_fov * aspect)
    ndc_y = camera_y / (camera_z * tan_half_fov)
    pixel_x = ((ndc_x + 1.0) * 0.5) * width - 0.5
    pixel_y = ((1.0 - ndc_y) * 0.5) * height - 0.5
    return pixel_x, pixel_y


def trace_frame(scene_state: dict[str, object], cube_center):
    width = scene_state["width"]
    height = scene_state["height"]
    camera = scene_state["camera"]
    plane = scene_state["scene"]["plane"]
    cube = scene_state["scene"]["cube"]
    plane_half_extent_xy = (
        plane["nominal_mesh_extent_xy"][0] * plane["scale"][0],
        plane["nominal_mesh_extent_xy"][1] * plane["scale"][1],
    )
    cube_half_extent = (
        cube["nominal_half_extent"][0] * cube["scale"][0],
        cube["nominal_half_extent"][1] * cube["scale"][1],
        cube["nominal_half_extent"][2] * cube["scale"][2],
    )

    hits = []
    for y in range(height):
        for x in range(width):
            origin, direction = camera_ray(camera, x, y, width, height)
            plane_hit = intersect_plane(origin, direction, tuple(plane["center"]), plane_half_extent_xy)
            cube_hit = intersect_aabb(origin, direction, cube_center, cube_half_extent)

            winner = None
            if plane_hit is not None:
                winner = plane_hit
            if cube_hit is not None and (winner is None or cube_hit["depth"] < winner["depth"]):
                winner = cube_hit
            if winner is None:
                winner = {
                    "kind": "none",
                    "point": vec_add(origin, vec_mul(direction, 1_000.0)),
                    "depth": 1.0,
                    "normal": (0.0, 0.0, 1.0),
                }
            hits.append(winner)
    return hits


def hits_to_normals(hits) -> list[list[float]]:
    return [[hit["normal"][0], hit["normal"][1], hit["normal"][2]] for hit in hits]


def derive_motion_and_masks(
    scene_state: dict[str, object],
    current_hits,
    previous_hits,
    current_color: list[list[float]],
    previous_color: list[list[float]],
):
    width = scene_state["width"]
    height = scene_state["height"]
    camera = scene_state["camera"]
    cube_state = scene_state["scene"]["cube"]
    previous_center = tuple(cube_state["previous_center"])
    current_center = tuple(cube_state["current_center"])
    cube_delta = vec_sub(current_center, previous_center)

    motion_vectors = []
    roi_mask = []
    disocclusion_mask = []
    for index, (current_hit, previous_hit, current_pixel, previous_pixel) in enumerate(
        zip(current_hits, previous_hits, current_color, previous_color)
    ):
        x = index % width
        y = index // width

        if current_hit["kind"] == "cube":
            previous_point = vec_sub(current_hit["point"], cube_delta)
            projected = project_point(camera, previous_point, width, height)
            if projected is None:
                motion = [0.0, 0.0]
            else:
                motion = [projected[0] - x, projected[1] - y]
        else:
            motion = [0.0, 0.0]

        motion_vectors.append(motion)

        changed_object = current_hit["kind"] != previous_hit["kind"]
        color_delta = max(abs(current_pixel[channel] - previous_pixel[channel]) for channel in range(3))
        moving_region = current_hit["kind"] == "cube" or previous_hit["kind"] == "cube"
        roi_mask.append(bool(moving_region or changed_object or color_delta > 0.05))
        disocclusion_mask.append(bool(current_hit["kind"] == "plane" and previous_hit["kind"] == "cube"))

    return motion_vectors, roi_mask, disocclusion_mask


def metadata_payload(scene_state: dict[str, object]) -> dict[str, object]:
    return {
        "frame_index": 1,
        "history_frame_index": 0,
        "width": scene_state["width"],
        "height": scene_state["height"],
        "source_kind": "unreal_native",
        "externally_validated": True,
        "real_external_data": True,
        "data_description": "Real Unreal SceneCapture exports with crate-local numeric materialization for DSFB replay",
        "provenance_label": "unreal_native",
        "scene_name": "DSFBTemporalCapture",
        "shot_name": "minimal_temporal_pair",
        "exposure": "manual_auto_exposure_disabled",
        "tonemap": "scene_capture_png_linearized",
        "camera": {
            "name": scene_state["camera"]["name"],
            "position": scene_state["camera"]["position"],
            "forward": scene_state["camera"]["forward"],
            "fov_degrees": scene_state["camera"]["fov_degrees"],
            "jitter_pixels": [0.0, 0.0],
        },
        "notes": [
            "Raw current_color and previous_color came from Unreal SceneCapture2D final-color PNG exports and were linearized into json_rgb_f32.",
            "Raw current_normals and previous_normals came from the Unreal WorldNormal visualization material and are retained under raw/ as provenance.",
            "current_normals.json and previous_normals.json are derived deterministically from the Unreal scene metadata and per-pixel geometry trace because the visualization PNG is not treated as a numerically stable unit-normal field on this editor-side Linux path.",
            "Raw current_depth and previous_depth came from the Unreal SceneDepth visualization material and were decoded into monotonic visualization depth scalars.",
            "motion_vectors.json was derived deterministically from Unreal camera/object metadata because the editor-side Linux path did not expose a stable dense velocity export for this minimal sample.",
            "roi_mask.json and disocclusion_mask.json were derived from the same Unreal scene metadata and exported frame pair to make the evidence bundle easier to audit.",
            "Do not relabel synthetic data as unreal_native.",
        ],
    }


def capture_log(scene_state: dict[str, object]) -> str:
    return f"""# DSFB Unreal-Native Capture Log

This sample bundle is the canonical crate-local Unreal-native example.

Raw export command:
- /home/one/Unreal/UE_5.7.2/Engine/Binaries/Linux/UnrealEditor crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject -ExecutePythonScript=crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py -stdout -FullStdOutLogOutput

Dataset materialization command:
- python3 crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py

Strict replay command:
- cargo run --release -- run-unreal-native --manifest examples/unreal_native_capture_manifest.json --output generated/unreal_native_runs

Resolution:
- {scene_state["width"]}x{scene_state["height"]}

Direct Unreal exports retained under raw/:
- previous_color.png
- current_color.png
- previous_normals.png
- current_normals.png
- previous_depth.png
- current_depth.png

Materialized replay buffers:
- previous_color.json
- current_color.json
- previous_normals.json              # derived from Unreal scene metadata
- current_normals.json               # derived from Unreal scene metadata
- previous_depth.json
- current_depth.json
- motion_vectors.json                # derived from Unreal scene metadata
- roi_mask.json
- disocclusion_mask.json
- metadata.json

Boundaries:
- motion_vectors.json is metadata-derived for this minimal sample because stable dense velocity export was not available on the editor-side Linux path used here
- current_normals.json and previous_normals.json are metadata-derived for this minimal sample because the WorldNormal PNG export is retained as visual evidence, not trusted as a numerically stable unit-normal field
- depth is labeled monotonic_visualized_depth rather than linear depth because the raw Unreal depth export is a visualization pass
- this is still an Unreal-native empirical replay path, not a synthetic substitute
"""


def main() -> None:
    frame = frame_dir()
    state_path = frame / "scene_state.json"
    if not state_path.exists():
        raise RuntimeError(
            f"{state_path} was missing; run the Unreal export script before building the dataset"
        )

    scene_state = load_json(state_path)
    width = int(scene_state["width"])
    height = int(scene_state["height"])

    previous_color_rgb = rgb_tuples(raw_dir() / "previous_color.png", width, height)
    current_color_rgb = rgb_tuples(raw_dir() / "current_color.png", width, height)
    previous_normals_rgb = rgb_tuples(raw_dir() / "previous_normals.png", width, height)
    current_normals_rgb = rgb_tuples(raw_dir() / "current_normals.png", width, height)
    previous_depth_rgb = rgb_tuples(raw_dir() / "previous_depth.png", width, height)
    current_depth_rgb = rgb_tuples(raw_dir() / "current_depth.png", width, height)

    previous_color = linearize_color(previous_color_rgb)
    current_color = linearize_color(current_color_rgb)
    previous_depth = decode_depth(previous_depth_rgb)
    current_depth = decode_depth(current_depth_rgb)

    validate_raw_exports(
        width,
        height,
        current_color,
        previous_color,
        current_depth,
        current_normals_rgb,
        previous_normals_rgb,
    )

    previous_hits = trace_frame(scene_state, tuple(scene_state["scene"]["cube"]["previous_center"]))
    current_hits = trace_frame(scene_state, tuple(scene_state["scene"]["cube"]["current_center"]))
    previous_normals = hits_to_normals(previous_hits)
    current_normals = hits_to_normals(current_hits)
    motion_vectors, roi_mask, disocclusion_mask = derive_motion_and_masks(
        scene_state,
        current_hits,
        previous_hits,
        current_color,
        previous_color,
    )

    write_json(frame / "previous_color.json", {"width": width, "height": height, "data": previous_color})
    write_json(frame / "current_color.json", {"width": width, "height": height, "data": current_color})
    write_json(frame / "previous_normals.json", {"width": width, "height": height, "data": previous_normals})
    write_json(frame / "current_normals.json", {"width": width, "height": height, "data": current_normals})
    write_json(frame / "previous_depth.json", {"width": width, "height": height, "data": previous_depth})
    write_json(frame / "current_depth.json", {"width": width, "height": height, "data": current_depth})
    write_json(frame / "motion_vectors.json", {"width": width, "height": height, "data": motion_vectors})
    write_json(frame / "roi_mask.json", {"width": width, "height": height, "data": roi_mask})
    write_json(
        frame / "disocclusion_mask.json",
        {"width": width, "height": height, "data": disocclusion_mask},
    )
    write_json(frame / "metadata.json", metadata_payload(scene_state), pretty=True)
    (frame / "capture_commands.txt").write_text(capture_log(scene_state), encoding="utf-8")

    print(f"Materialized Unreal-native dataset in {frame}")
    print(f"Canonical manifest: {manifest_path()}")


if __name__ == "__main__":
    main()
