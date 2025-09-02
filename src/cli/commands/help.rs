use rust_i18n::t;

/// Execute localized help command
pub fn execute() {
    println!("{}", t!("about"));
    println!();
    println!("{}:", t!("help.usage"));
    println!("  warp [OPTIONS] <COMMAND>");
    println!();
    println!("{}:", t!("help.commands"));
    println!("  law             {}", t!("commands.law"));
    println!("  ordinance       {}", t!("commands.ordinance"));
    println!("  precedent       {}", t!("commands.precedent"));
    println!("  admrule         {}", t!("commands.admrule"));
    println!("  interpretation  {}", t!("commands.interpretation"));
    println!("  search          {}", t!("commands.search"));
    println!("  config          {}", t!("commands.config"));
    println!("  cache           {}", t!("commands.cache"));
    println!("  version         {}", t!("commands.version"));
    println!("  completions     {}", t!("commands.completions"));
    println!();
    println!("{}:", t!("help.global_options"));
    println!("  -v, --verbose   {}", t!("options.verbose"));
    println!("  -q, --quiet     {}", t!("options.quiet"));
    println!("      --no-cache  {}", t!("options.no_cache"));
    println!("  -f, --format    {}", t!("options.format"));
    println!("      --lang      Set language (ko, en)");
    println!("  -h, --help      Print help");
    println!("  -V, --version   Print version");
    println!();
    println!("{}:", t!("help.examples"));
    println!("  warp law \"민법\"");
    println!("  warp --lang en law \"civil law\"");
    println!("  warp ordinance --region 서울 \"주택\"");
    println!("  warp precedent --court 대법원 \"손해배상\"");
}
