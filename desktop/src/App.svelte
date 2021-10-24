<script>
</script>

<main>
  <div class="header">
    MENU
  </div>
  <div class="page">
    <div class="page-content">
      HTML
    </div>
  </div>
</main>

<style>
  main {
    width: 100%;
    height: 100%;

    display: flex;
    flex-direction: column;
    flex-wrap: nowrap;
  }

  .header {
  }

  .page {
    display: flex;
    flex-direction: column;
    flex-grow: 1;
  }

  .page-content {
    flex-grow: 1;
    overflow: auto;
    min-height: 2em;
  }
</style>
