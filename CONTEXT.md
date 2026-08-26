# BDL Download Context

BDL turns Bilibili sources into durable local media downloads while keeping task planning and existing files predictable.

## Language

**Final output**:
A completed media file at the rendered destination path. In-progress transfer data is not a final output.
_Avoid_: Partial file, cache

**Skip existing**:
The default name-collision policy: do not create a new task when its final output already exists.
_Avoid_: Skip duplicate task

**Overwrite existing**:
A name-collision policy that deliberately replaces the final output at the rendered destination path.
_Avoid_: Resume, update

**Extend name**:
A name-collision policy that preserves the existing final output and chooses a numbered destination path for the new task.
_Avoid_: Rename existing
