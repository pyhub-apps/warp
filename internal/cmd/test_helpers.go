package cmd

import (
	"github.com/pyhub-apps/pyhub-warp-cli/internal/i18n"
	"github.com/spf13/cobra"
)

// createTestRootCommand creates a root command for testing
func createTestRootCommand() *cobra.Command {
	// Initialize i18n for testing (Korean by default)
	_ = i18n.Init()

	// Initialize root command
	initRootCmd()
	setupFlags()

	// Initialize and add subcommands
	initConfigCmd()
	initLawCmd()
	initSearchCmd()
	initPrecedentCmd()
	initAdmruleCmd()
	initInterpretationCmd()
	initOrdinanceCmd()
	initVersionCmd()

	// Build command hierarchy
	rootCmd.AddCommand(configCmd)
	rootCmd.AddCommand(lawCmd)
	rootCmd.AddCommand(searchCmd)
	rootCmd.AddCommand(precedentCmd)
	rootCmd.AddCommand(admruleCmd)
	rootCmd.AddCommand(interpretationCmd)
	rootCmd.AddCommand(ordinanceCmd)
	rootCmd.AddCommand(versionCmd)

	// Config subcommands
	if configCmd != nil {
		initConfigSetCmd()
		initConfigGetCmd()
		initConfigPathCmd()
		configCmd.AddCommand(configSetCmd)
		configCmd.AddCommand(configGetCmd)
		configCmd.AddCommand(configPathCmd)
	}

	// Law subcommands
	if lawCmd != nil {
		initLawSearchCmd()
		initLawDetailCmd()
		initLawHistoryCmd()
		lawCmd.AddCommand(lawSearchCmd)
		lawCmd.AddCommand(lawDetailCmd)
		lawCmd.AddCommand(lawHistoryCmd)
	}

	// Precedent subcommands
	if precedentCmd != nil {
		initPrecedentSearchCmd()
		initPrecedentDetailCmd()
		precedentCmd.AddCommand(precedentSearchCmd)
		precedentCmd.AddCommand(precedentDetailCmd)
	}

	// Admrule subcommands
	if admruleCmd != nil {
		initAdmruleSearchCmd()
		initAdmruleDetailCmd()
		admruleCmd.AddCommand(admruleSearchCmd)
		admruleCmd.AddCommand(admruleDetailCmd)
	}

	// Interpretation subcommands
	if interpretationCmd != nil {
		initInterpretationSearchCmd()
		initInterpretationDetailCmd()
		interpretationCmd.AddCommand(interpretationSearchCmd)
		interpretationCmd.AddCommand(interpretationDetailCmd)
	}

	// Ordinance subcommands
	if ordinanceCmd != nil {
		initOrdinanceSearchCmd()
		initOrdinanceDetailCmd()
		ordinanceCmd.AddCommand(ordinanceSearchCmd)
		ordinanceCmd.AddCommand(ordinanceDetailCmd)
	}

	return rootCmd
}
