from subprocess import Popen, PIPE


def main():
    p = Popen(["stockfish"], stdout=PIPE, stdin=PIPE, stderr=PIPE, text=True)
    res = p.communicate("uci")[0]
    print(res)
    opts = [("Threads", "8"), ("UCI_ShowWDL", "true")]
    cmd = "\n".join(f"setoption name {k} value {v}\n" for k, v in opts)
    p.communicate(cmd)


main()
