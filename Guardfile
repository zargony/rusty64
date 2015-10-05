$:.unshift File.expand_path('../.guard/lib', __FILE__)

guard :rust do
  watch(/^Cargo.toml$/)
  watch(/^src\/.+\.rs$/)
end
