{ config, pkgs, lib, ... }:
{
  options = {
    puterium =
      let
        submoduleEnum = spec: types.addCheck (types.submodule spec) (v: builtins.length (lib.attrsToList v));
        upstreamArg = mkOption {
          default = null;
          type = types.nullOr types.attrsOf types.enum [ "strong" "weak" ];
        };
        defaultOffArg = mkOption {
          default = null;
          type = types.nullOr types.bool;
        };
        envArg = _: mkOption {
          default = null;
          type = types.nullOr types.submodule {
            clear = mkOption {
              default = null;
              type = types.nullOr types.attrsOf types.bool;
            };
            add = mkOption {
              default = null;
              type = types.nullOr types.attrsOf types.str;
            };
          };
        };
        simpleDurationType = types.strMatching "\\d+[hms]";
        restartDelayArg = mkOption {
          default = null;
          type = types.nullOr simpleDurationType;
          description = "Like 10s or 5m";
        };
        stopTimeoutArg = mkOption {
          default = null;
          type = types.nullOr simpleDurationType;
          description = "Like 10s or 5m";
        };
        commandArg = mkOption {
          working_directory = mkOption {
            default = null;
            type = types.nullOr types.str;
          };
          environment = envArg;
          command = mkOption {
            type = types.listOf types.str;
          };
        };
      in
      {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Enable the puterium service for managing puterium services (tasks)";
        };
        environment = envArg;
        tasks = mkOption {
          description = "See puterium documentation for field details";
          type = types.attrsOf submoduleEnum {
            empty = mkOption {
              default = null;
              type = types.nullOr types.submodule {
                upstream = upstreamArg;
                default_off = defaultOffArg;
              };
            };
            perpetual = mkOption {
              default = null;
              type = types.nullOr types.submodule {
                upstream = upstreamArg;
                default_off = defaultOffArg;
                command = commandArg;
                started_check = mkOption {
                  default = null;
                  type = types.nullOr submoduleEnum {
                    tcp_socket = mkOption {
                      default = null;
                      type = types.nullOr types.str;
                    };
                    path = mkOption {
                      default = null;
                      type = types.nullOr types.str;
                    };
                  };
                };
                restart_delay = restartDelayArg;
                stop_timeout = stopTimeoutArg;
              };
            };
            finite = mkOption {
              default = null;
              type = types.nullOr types.submodule {
                upstream = upstreamArg;
                default_off = defaultOffArg;
                command = commandArg;
                success_codes = mkOption {
                  default = null;
                  type = types.nullOr types.listOf types.int;
                };
                started_action = mkOption {
                  default = null;
                  type = types.nullOr types.enum [ "turn_off" "delete" ];
                };
                restart_delay = restartDelayArg;
                stop_timeout = stopTimeoutArg;
              };
            };
            external = mkOption {
              default = null;
              type = types.nullOr types.str;
            };
          };
        };
      };
  };
  config =
    let
      cfg = config.volumesetup;
    in
    {
      systemd.services = lib.mkIf cfg.enable {
        volumesetup = {
          wantedBy = [ "multi-user.target" ];
          serviceConfig.Type = "simple";
          startLimitIntervalSec = 0;
          serviceConfig.Restart = "on-failure";
          serviceConfig.RestartSec = 60;
          script =
            let
              pkg = import ./package.nix;
              taskDirs = derivation {
                name = "puterium-task-configs";
                builder = "${pkgs.python3}/bin/python3";
                args = [
                  ./module_gendir.py
                  (builtins.toJSON (builtins.listToAttrs (builtins.concatMap (lib.attrsToList [
                    config.puterium.empty
                    config.puterium.perpetual
                    config.puterium.finite
                    config.puterium.external
                  ]))))
                ];
              };
            in
            lib.concatStringsSep " " [
              "${pkg}/bin/puterium"
              "demon"
              "run"
              (pkgs.writeText "puterium-config" (builtins.toJSON (builtins.listToAttrs (
                [ ]
                ++ (lib.option config.puterium.environment {
                  name = "environment";
                  value = config.puterium.environment;
                })
                ++ [{
                  name = "task_dirs";
                  value = taskDirs;
                }]
              ))))
            ];
        };
      };
    };
}
