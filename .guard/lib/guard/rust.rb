module Guard
  class Rust < Plugin
    def run_all
      puts "\n\n=== CARGO TEST (ALL)\n\n"
      system 'cargo test'
    end

    def run_on_changes (paths)
      # Extract module names from paths
      mods = paths.map do |path|
        %r{^src/(.+).rs$}.match(path) do |m|
          m[1].sub(%r{/(mod|tests)$}, '').sub(%r{^(lib|main)$}, '').gsub('/', '::')
        end
      end

      # Run tests of changed modules. Fall back to run all tests if a module
      # name could not be detected (like for Cargo.toml or other unknown files)
      if mods.any? { |m| m.to_s.empty? }
        run_all
      else
        puts "\n"
        mods.uniq.each do |mod|
          puts "\n=== CARGO TEST #{mod}\n\n"
          system "cargo test #{mod}"
        end
      end
    end
  end
end
