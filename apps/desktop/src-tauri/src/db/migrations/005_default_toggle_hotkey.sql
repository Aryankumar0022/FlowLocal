-- Make the default Ctrl+Space behavior tap-to-toggle.
-- This updates only the old shipped default, leaving custom hotkeys untouched.
UPDATE settings
SET value = '"toggle"', updated_at = unixepoch() * 1000
WHERE key = 'hotkey_mode'
  AND value = '"hold"'
  AND EXISTS (
      SELECT 1 FROM settings
      WHERE key = 'hotkey' AND value = '"ctrl+space"'
  );
