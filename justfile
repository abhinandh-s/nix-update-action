git-tag:
  git add .
  git commit -m "fix: some mdg"
  git push origin master
  # Force the tag again so the Action sees the change
  git tag -fa v1.0.0 -m "some msg"
  git push origin v1.0.0 --force
