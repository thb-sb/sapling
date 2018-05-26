import test_hgsubversion_util


class TestFetchBinaryFiles(test_hgsubversion_util.TestBase):
    stupid_mode_tests = True

    def test_binaryfiles(self):
        repo = self._load_fixture_and_fetch("binaryfiles.svndump")
        self.assertEqual("cce7fe400d8d", str(repo["tip"]))


if __name__ == "__main__":
    import silenttestrunner

    silenttestrunner.main(__name__)
