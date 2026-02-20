# Design: Move Auto-Generated Index Page

## Problem
Currently, Chisel automatically generates `index.md` in the documentation root, overwriting any user content. This prevents users from creating custom landing pages for their documentation, which is the default behavior in Starlight and many static site generators.

## Goal
Stop overwriting `index.md` with the auto-generated file list. Instead, generate a separate "Table of Contents" file (e.g., `contents.md`) and let the user manage `index.md`.

## Proposed Solution

1.  **New Output File**:
    *   Change `rebuild_index` to write the full document list to `contents.md` (or `reference.md`?).
    *   Let's call it `contents.md` to represent a "Table of Contents".

2.  **Migration Logic**:
    *   When `rebuild_index` runs (or during `init`):
        *   Check if `index.md` exists.
        *   Check if `index.md` contains the marker string: `"Automatically managed by Chisel."`.
        *   **If Yes (Auto-Generated):**
            *   Overwrite `index.md` with a new default template:
                ```markdown
                ---
                title: Welcome to Chisel Docs
                ---

                # Welcome to Chisel Docs

                This is your documentation home page. It is now safe to edit!

                - [View All Documents](contents)
                ```
            *   Generate `contents.md` with the full list.
        *   **If No (User-Curated):**
            *   Do NOT touch `index.md`.
            *   Generate `contents.md` with the full list.

3.  **Default `contents.md` Template**:
    *   The `contents.md` file will contain the auto-generated list of documents, similar to the current `index.md`.
    *   Title: "Table of Contents"
    *   Content:
        ```markdown
        ---
        title: Table of Contents
        ---

        # Table of Contents

        Automatically managed by Chisel.

        (List of docs...)
        ```

## Impact
- Users can now customize their documentation landing page.
- Existing auto-generated `index.md` files will be migrated to the new structure automatically on the next build/save.
- No data loss for users who have already customized `index.md` (as we check for the marker).

## Open Questions
- Should we use `contents.md` or `reference.md`? `contents.md` seems more neutral.
- Should we update `rebuild_index` to handle the migration, or a separate migration step? `rebuild_index` is called frequently, so checking the marker is cheap and robust.
