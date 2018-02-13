  $ . $TESTDIR/library.sh

setup configuration
  $ setup_config_repo
  $ cd $TESTTMP

setup common configuration
  $ cat >> $HGRCPATH <<EOF
  > [ui]
  > ssh="$DUMMYSSH"
  > EOF


setup repo

  $ hg init repo-hg

Init treemanifest and remotefilelog
  $ cd repo-hg
  $ cat >> .hg/hgrc <<EOF
  > [extensions]
  > treemanifest=
  > remotefilelog=
  > [treemanifest]
  > server=True
  > [remotefilelog]
  > server=True
  > shallowtrees=True
  > EOF

  $ touch a
  $ hg add a
  $ hg ci -ma
  $ hg log
  changeset:   0:3903775176ed
  tag:         tip
  user:        test
  date:        Thu Jan 01 00:00:00 1970 +0000
  summary:     a
  
  $ cd $TESTTMP

setup repo2
  $ cat >> $HGRCPATH <<EOF
  > [extensions]
  > remotefilelog=
  > [remotefilelog]
  > cachepath=$TESTTMP/cachepath
  > EOF
  $ hgcloneshallow ssh://user@dummy/repo-hg repo2 --noupdate
  requesting all changes
  adding changesets
  adding manifests
  adding file changes
  added 1 changesets with 0 changes to 0 files
  new changesets 3903775176ed

  $ cd repo2
  $ cat >> .hg/hgrc <<EOF
  > [extensions]
  > treemanifest=
  > remotefilelog=
  > [treemanifest]
  > server=False
  > treeonly=True
  > [remotefilelog]
  > server=False
  > reponame=repo
  > EOF
  $ hg pull
  pulling from ssh://user@dummy/repo-hg
  searching for changes
  no changes found

  $ cd $TESTTMP
  $ cd repo-hg
  $ touch b
  $ hg add b
  $ hg ci -mb
  $ echo content > c
  $ hg add c
  $ hg ci -mc
  $ mkdir dir
  $ echo 1 > dir/1
  $ mkdir dir2
  $ echo 2 > dir/2
  $ hg addremove
  adding dir/1
  adding dir/2
  $ hg ci -m 'new directory'
  $ echo cc > c
  $ hg addremove
  $ hg ci -m 'modify file'
  $ hg mv dir/1 dir/rename
  $ hg ci -m 'rename'
  $ hg debugdrawdag <<'EOS'
  >   D  # D/D=1\n2\n
  >  /|  # B/D=1\n
  > B C  # C/D=2\n
  > |/   # A/D=x\n
  > A
  > EOS
  $ hg log --graph -T '{node|short} {desc}'
  o    e635b24c95f7 D
  |\
  | o  d351044ef463 C
  | |
  o |  9a827afb7e25 B
  |/
  o  af6aa0dfdf3d A
  
  @  28468743616e rename
  |
  o  329b10223740 modify file
  |
  o  a42a44555d7c new directory
  |
  o  3e19bf519e9a c
  |
  o  0e067c57feba b
  |
  o  3903775176ed a
  
  $ cd ..
  $ blobimport --blobstore files --linknodes repo-hg repo

blobimport currently doesn't handle bookmarks, but server requires the directory.
  $ mkdir -p repo/books

Need a place for the socket to live
  $ mkdir -p repo/.hg

start mononoke

  $ mononoke -P $TESTTMP/mononoke-config -B test-config
  $ wait_for_mononoke $TESTTMP/repo
  $ hgmn debugwireargs ssh://user@dummy/repo one two --three three
  one two three None None

  $ cd repo2
  $ hg up 0
  1 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ hgmn pull ssh://user@dummy/repo --traceback
  pulling from ssh://user@dummy/repo
  searching for changes
  adding changesets
  adding manifests
  adding file changes
  added 9 changesets with 0 changes to 0 files (+1 heads)
  new changesets af6aa0dfdf3d:28468743616e
  (run 'hg heads' to see heads, 'hg merge' to merge)

  $ hg log -r '3903775176ed::329b10223740' --graph  -T '{node|short} {desc}'
  o  329b10223740 modify file
  |
  o  a42a44555d7c new directory
  |
  o  3e19bf519e9a c
  |
  o  0e067c57feba b
  |
  @  3903775176ed a
  
  $ ls
  a
  $ hgmn --config paths.default=ssh://user@dummy/repo up 28468743616e
  4 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ ls
  a
  b
  c
  dir
  $ cat c
  cc
  $ hgmn --config paths.default=ssh://user@dummy/repo up 28468743616e
  0 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ hg log c -T '{node|short} {desc}\n'
  warning: file log can be slow on large repos - use -f to speed it up
  329b10223740 modify file
  3e19bf519e9a c
  $ cat dir/rename
  1
  $ cat dir/2
  2
  $ hg log dir/rename -f -T '{node|short} {desc}\n'
  
  $ hg st --change 28468743616e -C
  A dir/rename
    dir/1
  R dir/1

  $ hgmn --config paths.default=ssh://user@dummy/repo up e635b24c95f7
  4 files updated, 0 files merged, 5 files removed, 0 files unresolved
Sort the output because the output is unpredictable because of merges
  $ hg log D --follow -T '{node|short} {desc}\n' | sort
  e635b24c95f7 D
  d351044ef463 C
  9a827afb7e25 B
  af6aa0dfdf3d A

Create a new bookmark and try and send it over the wire
Test commented while we have no bookmark support in blobimport or easy method
to create a fileblob bookmark
#  $ cd ../repo
#  $ hg bookmark test-bookmark
#  $ hg bookmarks
#   * test-bookmark             0:3903775176ed
#  $ cd ../repo2
#  $ hgmn pull ssh://user@dummy/repo
#  pulling from ssh://user@dummy/repo
#  searching for changes
#  no changes found
#  adding remote bookmark test-bookmark
#  $ hg bookmarks
#     test-bookmark             0:3903775176ed
