# BDL Download Context

BDL turns Bilibili sources into durable local media downloads while keeping task planning and existing files predictable.

## Language

### Content

**Source**:
A Bilibili input that BDL resolves into a browsable set of items and parts.
_Avoid_: URL, page

**Item**:
A titled content entry inside a source. An item contains one or more parts.
_Avoid_: Source, task

**Part**:
The smallest independently selectable media unit of an item, such as a video P or an episode.
_Avoid_: Item, task

### Downloads

**Download task**:
A durable request to turn one selected part into one final output.
_Avoid_: Job, queue row

**Download resource**:
An input needed by a download task, such as a video stream, audio stream, or selected auxiliary asset.
_Avoid_: Final output, file

**Final output**:
A completed media file at the rendered destination path. In-progress transfer data is not a final output.
_Avoid_: Partial file, cache

**Duplicate task**:
A proposed download task whose selected part matches a task already in transfer history. This is independent of whether a final output exists.
_Avoid_: Existing output, name collision

### Name collisions

**Skip existing**:
The default name-collision policy: do not create a new task when its final output already exists.
_Avoid_: Skip duplicate task

**Overwrite existing**:
A name-collision policy that deliberately replaces the final output at the rendered destination path.
_Avoid_: Resume, update

**Extend name**:
A name-collision policy that preserves the existing final output and chooses a numbered destination path for the new task.
_Avoid_: Rename existing
