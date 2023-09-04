import { NextPage } from "next";
import Head from "next/head";
import { rspc } from "../src/rspc";
import styles from "../styles/Home.module.css";

const UsingUseQuery: NextPage = () => {
  const { data, isLoading, error } = rspc.useQuery(["echo", "Hello!"]);

  return (
    <div className={styles.container}>
      <Head>
        <title>Using useQuery | RSPC Example with Next.js</title>
      </Head>

      <main className={styles.main}>
        <h1 className={styles.title}>
          <code>useQuery</code>
        </h1>
        <p className={styles.description}>
          {isLoading && "Loading data ..."}
          {data && `RSPC says: ${data}`}
          {error && JSON.stringify(error)}
        </p>
      </main>
    </div>
  );
};

export default UsingUseQuery;
