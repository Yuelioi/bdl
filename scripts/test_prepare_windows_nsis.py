import hashlib
import importlib.util
import io
from pathlib import Path
import tempfile
import sys
import unittest
import zipfile

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("nsis", Path(__file__).with_name("prepare-windows-nsis.py"))
nsis = importlib.util.module_from_spec(spec)
spec.loader.exec_module(nsis)


class InstallerDownloadTests(unittest.TestCase):
    def setUp(self):
        artifacts = Path(__file__).resolve().parent.parent / ".task"
        artifacts.mkdir(parents=True, exist_ok=True)
        self.folder = tempfile.TemporaryDirectory(prefix="nsis-test-", dir=artifacts)
        self.addCleanup(self.folder.cleanup)
        self.root = Path(self.folder.name)

    def test_timeout_and_corrupt_response_retry_before_publishing_verified_file(self):
        attempts = []
        def opener(url, timeout):
            attempts.append((url, timeout))
            if len(attempts) == 1:
                raise TimeoutError()
            return io.BytesIO(b"bad" if len(attempts) == 2 else b"verified")
        output = self.root / "plugin.dll"
        nsis.download("https://example.invalid/plugin", output, hashlib.sha1(b"verified").hexdigest(), opener, lambda _: None)
        self.assertEqual(len(attempts), 3)
        self.assertEqual(output.read_bytes(), b"verified")
        self.assertFalse(output.with_name("plugin.dll.download").exists())

    def test_exhausted_retries_keep_existing_file_and_remove_partial_download(self):
        output = self.root / "plugin.dll"
        output.write_bytes(b"existing")
        attempts = []
        def opener(url, timeout):
            attempts.append(url)
            raise TimeoutError()
        with self.assertRaises(TimeoutError):
            nsis.download("https://example.invalid/plugin", output, "wrong", opener, lambda _: None)
        self.assertEqual(len(attempts), 3)
        self.assertEqual(output.read_bytes(), b"existing")
        self.assertFalse(output.with_name("plugin.dll.download").exists())

    def test_archive_paths_cannot_escape_the_tool_directory(self):
        archive = self.root / "tools.zip"
        with zipfile.ZipFile(archive, "w") as bundle:
            bundle.writestr("nsis-3.11/../../escaped", b"bad")
        with self.assertRaises(ValueError):
            nsis.extract(archive, self.root / "NSIS")
        self.assertFalse((self.root / "escaped").exists())


if __name__ == "__main__":
    unittest.main()
