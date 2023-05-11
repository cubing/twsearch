#include "shell.h"

static void             cleanwhitespaces(char *line);
static int              parseline(char *line, char **v);

bool
checkfiles(void)
{
	/* TODO: add more checks (other files, use checksum...) */
	FILE *f;
	char fname[strlen(tabledir)+100];
	int i;

	for (i = 0; all_pd[i] != NULL; i++) {
		strcpy(fname, tabledir);
		strcat(fname, "/");
		strcat(fname, all_pd[i]->filename);
		if ((f = fopen(fname, "rb")) == NULL)
			return false;
		else
			fclose(f);
	}

	return true;
}

static void
cleanwhitespaces(char *line)
{
	char *i;

	for (i = line; *i != 0; i++)
		if (*i == '\t' || *i == '\n')
			*i = ' ';
}

/* This function assumes that **v is large enough */
static int
parseline(char *line, char **v)
{
	char *t;
	int n = 0;
	
	cleanwhitespaces(line);

	for (t = strtok(line, " "); t != NULL; t = strtok(NULL, " "))
		strcpy(v[n++], t);

	return n;
}

void
exec_args(int c, char **v)
{
	int i;
	char line[MAXLINELEN];
	Command *cmd = NULL;
	CommandArgs *args;
	Alg *scramble;

	for (i = 0; commands[i] != NULL; i++)
		if (!strcmp(v[0], commands[i]->name))
			cmd = commands[i];

	if (cmd == NULL) {
		fprintf(stderr, "%s: command not found\n", v[0]);
		goto exec_args_end;
	}

	args = cmd->parse_args(c-1, &v[1]);
	if (!args->success) {
		fprintf(stderr, "usage: %s\n", cmd->usage);
		goto exec_args_end;
	}

	if (args->scrstdin) {
		while (true) {
			if (fgets(line, MAXLINELEN, stdin) == NULL) {
				clearerr(stdin);
				break;
			}
			
			scramble = new_alg(line);

			printf(">>> Line: %s", line);

			if (scramble != NULL && scramble->len > 0) {
				args->scramble = scramble;
				cmd->exec(args);
				free_alg(scramble);
				args->scramble = NULL;
			} 
		}
	} else {
		cmd->exec(args);
	}

exec_args_end:
	free_args(args);
}

void
launch(bool batchmode)
{
	int i, shell_argc;
	char line[MAXLINELEN], **shell_argv;

	shell_argv = malloc(MAXNTOKENS * sizeof(char *));
	for (i = 0; i < MAXNTOKENS; i++)
		shell_argv[i] = malloc((MAXTOKENLEN+1) * sizeof(char));

	if (!batchmode) {
		fprintf(stderr, "Welcome to Nissy "VERSION".\n"
				"Type \"commands\" for a list of commands.\n");
	}

	while (true) {
		if (!batchmode) {
			fprintf(stdout, "nissy-# ");
		}

		if (fgets(line, MAXLINELEN, stdin) == NULL)
			break;

		if (batchmode) {
			printf(">>>\n"
			       ">>> Executing command: %s"
			       ">>>\n", line);
		}

		shell_argc = parseline(line, shell_argv);

		if (shell_argc > 0)
			exec_args(shell_argc, shell_argv);
	}

	for (i = 0; i < MAXNTOKENS; i++)
		free(shell_argv[i]);
	free(shell_argv);
}

int
main(int argc, char *argv[])
{
	char *closing_cmd[1] = { "freemem" };

	init_env();

	if (!checkfiles()) {
		fprintf(stderr,
			"--- Warning ---\n"
			"Some pruning tables are missing or unreadable\n"
			"You can generate them with `nissy gen'.\n"
			"---------------\n\n"
		);
	}

	if (argc > 1) {
		if (!strcmp(argv[1], "-b")) {
			launch(true);
		} else {
			exec_args(argc-1, &argv[1]);
		}
	} else {
		launch(false);
	}

	exec_args(1, closing_cmd);

	return 0;
}
