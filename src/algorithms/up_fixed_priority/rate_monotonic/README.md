# UniProcessor Rate Monotonic

[**🏠 Go back to all analyses**](../../../../README.md#-available-analyses)

### 🧪 Implemented Analyses

---

- **Classic** - *Liu & Layland* (1973) [^1]

    Preconditions:
    - Periodic Tasks
    - Implicit deadlines
    - Taskset sorted by period

    Worst-Case Complexity: *O(n)*

- **Classic Simple** - *Liu & Layland* (1973) [^1], i.e. worst case bound for any number of tasks

    Preconditions:
    - Periodic Tasks
    - Implicit deadlines
    - Taskset sorted by period

    Worst-Case Complexity: *O(n)*

- **Hyperbolic Bound** - *Bini, Buttazzo and Buttazzo* (2001) [^2]

    Preconditions:
    - Periodic Tasks
    - Implicit deadlines
    - Taskset sorted by period

    Worst-Case Complexity: *O(n)*

### 📚 References

[^1]: C. L. Liu and J. W. Layland, “Scheduling Algorithms for Multiprogramming in a Hard-Real-Time Environment,” J. ACM, vol. 20, no. 1, pp. 46–61, Jan. 1973, doi: 10.1145/321738.321743.

[^2]: E. Bini, G. Buttazzo, and G. Buttazzo, “A hyperbolic bound for the rate monotonic algorithm,” in Proceedings 13th Euromicro Conference on Real-Time Systems, June 2001, pp. 59–66. doi: 10.1109/EMRTS.2001.934000.