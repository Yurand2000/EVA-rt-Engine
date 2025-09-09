# Real-Time Analysis

## 🚀 Quick Start

### Prerequisites

- **Rust**: ≥1.?.?-nightly

<!-- ### Installation

### Basic Usage

### Advanced Usage -->

## 🔬 Available Analyses

### Single Processor Analyses

*References are available at the end of the page*

- **Rate Monotonic**
    - *Classic* - Liu & Layland (1973) [1]
    - *Hyperbolic Bound* - Bini, Buttazzo and Buttazzo (2001) [2]
- **Earliest Deadline First** - Liu & Layland (1973) [1]
- **Deadline Monotonic** - Leung & Whitehead (1982) [3]
- **Response Time Analysis** - Joseph & Pandya (1986) [4]

### Multi Processor Analyses

- **Global Deadline Monotonic** - Bertogna, Cirinei and Lipari (2005) [5]
- **Global Earliest Deadline First**
    - *Implicit Deadlines* - Goossens, Funk and Baruah (2003) [6]
    - *Constrained Deadlines* - Bertogna, Cirinei and Lipari (2009) [7]

## 📄 License

This project is licensed under the GNU General Public License v3 - see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Real-Time Analysis** was developed by:
- **Yuri Andriaccio** [yurand2000@gmail.com](mailto:yurand2000@gmail.com), [GitHub](https://github.com/Yurand2000).

## 📝 TODO - Future Work

## 📚 References
1. C. L. Liu and J. W. Layland, “Scheduling Algorithms for Multiprogramming in a Hard-Real-Time Environment,” J. ACM, vol. 20, no. 1, pp. 46–61, Jan. 1973, doi: 10.1145/321738.321743.
2. E. Bini, G. Buttazzo, and G. Buttazzo, “A hyperbolic bound for the rate monotonic algorithm,” in Proceedings 13th Euromicro Conference on Real-Time Systems, June 2001, pp. 59–66. doi: 10.1109/EMRTS.2001.934000.
3. J. Y.-T. Leung and J. Whitehead, “On the complexity of fixed-priority scheduling of periodic, real-time tasks,” Performance Evaluation, vol. 2, no. 4, pp. 237–250, Dec. 1982, doi: 10.1016/0166-5316(82)90024-4.
4. M. Joseph and P. Pandya, “Finding Response Times in a Real-Time System,” Comput J, vol. 29, no. 5, pp. 390–395, 1986, doi: 10.1093/comjnl/29.5.390.
5. M. Bertogna, M. Cirinei, and G. Lipari, “New Schedulability Tests for Real-Time Task Sets Scheduled by Deadline Monotonic on Multiprocessors,” in Principles of Distributed Systems, J. H. Anderson, G. Prencipe, and R. Wattenhofer, Eds., Berlin, Heidelberg: Springer, Dec. 2005, pp. 306–321. doi: 10.1007/11795490_24.
6. J. Goossens, S. Funk, and S. Baruah, “Priority-Driven Scheduling of Periodic Task Systems on Multiprocessors,” Real-Time Systems, vol. 25, no. 2, pp. 187–205, Sept. 2003, doi: 10.1023/A:1025120124771.
7. M. Bertogna, M. Cirinei, and G. Lipari, “Schedulability Analysis of Global Scheduling Algorithms on Multiprocessor Platforms,” IEEE Transactions on Parallel and Distributed Systems, vol. 20, no. 4, pp. 553–566, Apr. 2009, doi: 10.1109/TPDS.2008.129.

---

**Real-Time Analysis**